# Plan 056: Flow-Sensitive UAF Filtering

## Problem Statement

The SVFG-based Use-After-Free (UAF) checker produces 134 false positives on SV-COMP memsafety benchmarks. The root cause is that SVFG captures value-flow but not temporal ordering.

Example from `memsafety-broom/dll.c`:
```c
while (l) {
    struct dll *next = l->next;  // T1: Read BEFORE free
    free(l);                      // T2: Free
    l = next;                     // T3: Use next (read at T1, not T2)
}
```

The SVFG shows `l` flows to `free()` and `l->next` produces a value used later. The UAF checker reports this as a bug because it sees "free reaches use" on SVFG, but doesn't understand the load happened BEFORE the free.

## Solution Overview

Add program point tracking to SVFG and filter UAF findings based on temporal ordering:

1. **ProgramPointMap**: Track where each ValueId is defined (function, block, instruction index)
2. **Temporal ordering query**: Determine if one program point can execute after another
3. **Post-filter**: After finding UAF candidates, filter out those where "use" happens before "free"

## Detailed Design

### 1. ProgramPoint Type

```rust
/// A specific location in the program.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProgramPoint {
    pub function: FunctionId,
    pub block: BlockId,
    pub inst_index: u32,
}

/// Map from ValueId to its defining program point.
#[derive(Debug, Clone, Default)]
pub struct ProgramPointMap {
    points: BTreeMap<ValueId, ProgramPoint>,
}
```

Location: `crates/saf-analysis/src/svfg/program_point.rs`

### 2. ProgramPointMap API

```rust
impl ProgramPointMap {
    /// Insert a program point for a value.
    pub fn insert(&mut self, value: ValueId, point: ProgramPoint);

    /// Get the program point for a value.
    pub fn get(&self, value: ValueId) -> Option<ProgramPoint>;

    /// Check if `later` can execute after `earlier` in program order.
    pub fn can_happen_after(
        &self,
        earlier: ProgramPoint,
        later: ProgramPoint,
        cfgs: &BTreeMap<FunctionId, Cfg>,
    ) -> bool {
        // Different functions: conservatively return true
        if earlier.function != later.function {
            return true;
        }

        // Same block: compare instruction indices
        if earlier.block == later.block {
            return later.inst_index > earlier.inst_index;
        }

        // Different blocks: check CFG reachability
        if let Some(cfg) = cfgs.get(&earlier.function) {
            return cfg.is_reachable(earlier.block, later.block);
        }
        true // Conservative fallback
    }
}
```

### 3. CFG Reachability

Add to `crates/saf-analysis/src/cfg/mod.rs`:

```rust
impl Cfg {
    /// Check if `target` block is reachable from `source` block.
    pub fn is_reachable(&self, source: BlockId, target: BlockId) -> bool {
        let mut visited = BTreeSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(source);

        while let Some(block) = queue.pop_front() {
            if block == target { return true; }
            if !visited.insert(block) { continue; }

            if let Some(succs) = self.successors(block) {
                queue.extend(succs.iter().copied());
            }
        }
        false
    }
}
```

### 4. SVFG Builder Changes

Modify `SvfgBuilder::build()` to return `(Svfg, ProgramPointMap)`:

```rust
pub fn build(self) -> (Svfg, ProgramPointMap) {
    let mut graph = Svfg::new();
    let mut program_points = ProgramPointMap::new();

    // Build program point map during instruction iteration
    for func in &self.module.functions {
        if func.is_declaration { continue; }
        for block in &func.blocks {
            for (inst_idx, inst) in block.instructions.iter().enumerate() {
                let pp = ProgramPoint {
                    function: func.id,
                    block: block.id,
                    inst_index: inst_idx as u32,
                };
                if let Some(dst) = inst.dst {
                    program_points.insert(dst, pp);
                }
                // Track operands for parameters/globals
                for &operand in &inst.operands {
                    program_points.entry(operand).or_insert(pp);
                }
            }
        }
    }

    // ... existing 4-phase build ...

    (graph, program_points)
}
```

### 5. Temporal Filter Integration

Add to `crates/saf-analysis/src/checkers/pathsens_runner.rs`:

```rust
/// Filter out UAF findings where the "use" happens before the "free".
pub fn filter_temporal_infeasible(
    findings: Vec<CheckerFinding>,
    program_points: &ProgramPointMap,
    cfgs: &BTreeMap<FunctionId, Cfg>,
) -> Vec<CheckerFinding> {
    findings.into_iter()
        .filter(|f| {
            // Only apply temporal filter to UAF checker
            if f.checker_name != "use-after-free" {
                return true;
            }

            // Get program points for source (free) and sink (use)
            let src_vid = f.source_node.as_value();
            let sink_vid = f.sink_node.as_value();

            let (Some(src_vid), Some(sink_vid)) = (src_vid, sink_vid) else {
                return true; // Can't determine, keep conservatively
            };

            let (Some(src_pp), Some(sink_pp)) = (
                program_points.get(src_vid),
                program_points.get(sink_vid),
            ) else {
                return true; // Can't determine, keep conservatively
            };

            // Keep finding only if use can happen AFTER free
            program_points.can_happen_after(src_pp, sink_pp, cfgs)
        })
        .collect()
}
```

### 6. AnalysisContext Changes

Update `crates/saf-bench/src/svcomp/property.rs` to cache and propagate `ProgramPointMap`:

```rust
pub struct AnalysisContext<'a> {
    // ... existing fields ...
    svfg: OnceCell<Svfg>,
    program_points: OnceCell<ProgramPointMap>,  // NEW
}

impl<'a> AnalysisContext<'a> {
    pub fn svfg_with_points(&self) -> (&Svfg, &ProgramPointMap) {
        // Build both together, cache both
    }
}
```

## Implementation Steps

1. **Create `ProgramPoint` and `ProgramPointMap`** types
   - New file: `crates/saf-analysis/src/svfg/program_point.rs`
   - Add module to `crates/saf-analysis/src/svfg/mod.rs`

2. **Add `Cfg::is_reachable()`** method
   - Edit: `crates/saf-analysis/src/cfg/mod.rs`
   - Add tests

3. **Modify `SvfgBuilder::build()`** to return `(Svfg, ProgramPointMap)`
   - Edit: `crates/saf-analysis/src/svfg/builder.rs`
   - Update all callers

4. **Add `filter_temporal_infeasible()`** function
   - Edit: `crates/saf-analysis/src/checkers/pathsens_runner.rs`

5. **Integrate into memsafety analyzer**
   - Edit: `crates/saf-bench/src/svcomp/property.rs`
   - Pass CFGs and program points to temporal filter

6. **Update Python bindings** if needed
   - Edit: `crates/saf-python/src/svfg.rs`

7. **Test and benchmark**
   - Run `make test`
   - Run memsafety benchmarks, verify FalseIncorrect reduction

## Expected Impact

- **Target**: Reduce UAF false positives from 134 to ~20-40 (same-function cases)
- **Scope**: Only affects UAF checker; other checkers unchanged
- **Risk**: Low - post-filter approach doesn't change core SVFG or solver

## Future Extensions

- Interprocedural temporal ordering (across function calls)
- Apply to other checkers (null-deref with null-check ordering)
- Cache CFG reachability for performance
