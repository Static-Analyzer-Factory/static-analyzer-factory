# Plan 055: Memsafety Precision Improvements

## Problem Summary

SV-COMP memsafety benchmark shows 79 incorrect FALSE verdicts (false positives) due to:
1. Null-check branches not acting as sanitizers
2. Load/Store operations not classified as Dereference sinks
3. Guard extraction gaps (MemPhi, interprocedural, constants)

Current: -1790 score (79 FP, 20 incorrect TRUE, 110 FALSE total)
Target: Positive score (reduce FP by 80%+)

## Phase 1: Null-Check Sanitizers (~200 LOC)

### Problem
The `null_deref()` checker has `sanitizers: vec![]`. Code like:
```c
int *p = malloc(sizeof(int));
if (p != NULL) {
    *p = 42;  // Currently reported as null-deref, but guarded!
}
```

### Solution
Add a new sanitizer pattern that recognizes null-check branches in the SVFG.

**Option A: Branch-Based Sanitizer (Recommended)**
- Add `SitePattern::NullCheckGuard` that matches when:
  1. Value flows through a CondBr block
  2. The CondBr condition is `icmp eq/ne %val, null`
  3. We're on the "not-null" branch (else for eq, then for ne)

**Option B: Dominance-Based Sanitizer**
- Track which values are guarded by null checks via dominator analysis
- Add guard information to SVFG nodes

### Files to Modify
- `crates/saf-analysis/src/checkers/spec.rs` - Add `NullCheckBranch` pattern
- `crates/saf-analysis/src/checkers/site_classifier.rs` - Classify null-check branches
- `crates/saf-analysis/src/checkers/solver.rs` - Prune paths through null-check sanitizers

### Implementation

```rust
// In spec.rs
pub enum SitePattern {
    // ... existing patterns ...

    /// Null-check branch (icmp eq/ne ptr, null followed by CondBr).
    /// The sanitized path is the one where ptr is known non-null.
    NullCheckBranch,
}

// Update null_deref() spec
pub fn null_deref() -> CheckerSpec {
    CheckerSpec {
        // ...
        sanitizers: vec![SitePattern::NullCheckBranch],
    }
}
```

```rust
// In site_classifier.rs - add null-check detection
fn classify_null_checks(module: &AirModule, svfg: &Svfg) -> Vec<SvfgNodeId> {
    let mut sanitizers = Vec::new();

    for func in &module.functions {
        for block in &func.blocks {
            // Find ICmp instructions comparing to null
            for inst in &block.instructions {
                if let Operation::BinaryOp { kind } = &inst.op {
                    if matches!(kind, BinaryOp::ICmpEq | BinaryOp::ICmpNe) {
                        // Check if one operand is null constant
                        if is_null_operand(&inst.operands, module) {
                            // This is a null check - the non-null branch sanitizes
                            if let Some(dst) = inst.dst {
                                sanitizers.push(SvfgNodeId::value(dst));
                            }
                        }
                    }
                }
            }
        }
    }

    sanitizers
}
```

### Expected Impact
- 60-70% reduction in null-deref false positives
- Programs with proper null checks will no longer report false alarms

---

## Phase 2: Load/Store Dereference Classification (~150 LOC)

### Problem
The Dereference role only covers function calls (`memcpy(ptr, ...)`), not actual pointer dereferences in IR:
```llvm
%4 = load ptr, ptr %2     ; Load the pointer
store i32 7, ptr %4       ; Dereference it - NOT classified as sink!
```

### Solution
Classify Load and Store operations as Dereference sinks when they dereference a pointer.

### Files to Modify
- `crates/saf-analysis/src/checkers/site_classifier.rs` - Add Load/Store classification
- `crates/saf-analysis/src/checkers/spec.rs` - Add `LoadDeref` and `StoreDeref` patterns

### Implementation

```rust
// In site_classifier.rs - classify() function
Operation::Load => {
    // The pointer operand is being dereferenced
    if let Some(&ptr_val) = inst.operands.first() {
        let site = ClassifiedSite {
            inst_id: inst.id,
            func_id: func.id,
            block_id: block.id,
            callee_name: "__deref_load".to_string(),
            return_value: inst.dst,
            arguments: vec![SvfgNodeId::value(ptr_val)],
            roles: BTreeSet::from([ResourceRole::Dereference]),
        };
        sites.push(site);
    }
}

Operation::Store => {
    // operands: [value, ptr] - ptr is being dereferenced
    if let Some(&ptr_val) = inst.operands.get(1) {
        let site = ClassifiedSite {
            inst_id: inst.id,
            func_id: func.id,
            block_id: block.id,
            callee_name: "__deref_store".to_string(),
            return_value: None,
            arguments: vec![SvfgNodeId::value(ptr_val)],
            roles: BTreeSet::from([ResourceRole::Dereference]),
        };
        sites.push(site);
    }
}
```

### Expected Impact
- Enables null-deref detection for actual pointer dereferences (not just function calls)
- Combined with Phase 1 sanitizers, dramatically improves precision

---

## Phase 3: MemPhi Guard Bridging (~100 LOC)

### Problem
When SVFG trace contains MemPhi nodes, guard extraction skips them:
```
Trace: Value(a) → MemPhi → Value(b)
Guard extraction: only looks at Value nodes, skips MemPhi
Result: Guard between blocks is lost
```

### Solution
Bridge guards across MemPhi nodes by tracking the block location of MemPhi's predecessors.

### Files to Modify
- `crates/saf-analysis/src/z3_utils/guard.rs` - Enhance `extract_guards()`

### Implementation

```rust
// In guard.rs - extract_guards()
pub fn extract_guards(trace: &[SvfgNodeId], index: &ValueLocationIndex) -> PathCondition {
    let mut guards = Vec::new();
    let mut last_value_block: Option<(FunctionId, BlockId)> = None;

    for node in trace {
        match node {
            SvfgNodeId::Value(vid) => {
                if let Some(loc) = index.block_of(*vid) {
                    // Check guard from last value's block to this block
                    if let Some((func_id, prev_block)) = last_value_block {
                        if prev_block != loc.1 && func_id == loc.0 {
                            // Extract guard if crossing block boundary
                            if let Some(guard) = extract_block_guard(prev_block, loc.1, index) {
                                guards.push(guard);
                            }
                        }
                    }
                    last_value_block = Some(loc);
                }
            }
            SvfgNodeId::MemPhi(_) => {
                // Don't update last_value_block - bridge across MemPhi
            }
        }
    }

    PathCondition { guards }
}
```

### Expected Impact
- 10-20% reduction in false positives from memory operations
- Guards on memory stores/loads will be properly tracked

---

## Phase 4: Constant Resolution in Z3 (~100 LOC)

### Problem
Constants are treated as fresh Z3 variables, missing obvious contradictions:
```c
if (x == 5) {
    if (x == 6) {
        use(x);  // Can't be proven infeasible - Z3 sees (x == c1) && (x == c2)
    }
}
```

### Solution
Resolve constant operands in `resolve_operand()` to actual values.

### Files to Modify
- `crates/saf-analysis/src/z3_utils/guard.rs` - Enhance `resolve_operand()`

### Implementation

```rust
// In guard.rs
pub fn resolve_operand(vid: ValueId, module: &AirModule) -> OperandInfo {
    // Check if this ValueId corresponds to a constant
    if let Some(constant) = module.constants.get(&vid) {
        match constant {
            Constant::Int(val) => return OperandInfo::Constant(*val as i64),
            Constant::Null => return OperandInfo::Null,
            _ => {}
        }
    }

    // Fall back to symbolic value
    OperandInfo::Value(vid)
}
```

### Expected Impact
- 5-10% reduction in false positives from numeric comparisons
- Contradictory branch conditions will be detected

---

## Phase 5: Interprocedural Guard Propagation (~200 LOC)

### Problem
Guards from caller functions are lost at call sites:
```c
void callee(int *p) {
    *p = 42;  // Finding reported, but caller checked null!
}

void caller() {
    int *p = malloc(...);
    if (p != NULL) {
        callee(p);  // Guard not propagated to callee
    }
}
```

### Solution
Propagate caller guards to callee entry via function summaries.

### Files to Modify
- `crates/saf-analysis/src/z3_utils/guard.rs` - Add interprocedural guard tracking
- `crates/saf-analysis/src/checkers/pathsens_runner.rs` - Use caller context

### Implementation Sketch

```rust
// Build caller guard context per call site
pub struct CallerGuardContext {
    /// Guards active at each call site
    call_site_guards: BTreeMap<InstId, Vec<Guard>>,
}

impl CallerGuardContext {
    pub fn build(module: &AirModule, callgraph: &CallGraph) -> Self {
        // For each call site, collect dominating guards
        // ...
    }

    pub fn guards_at_call(&self, call_inst: InstId) -> &[Guard] {
        self.call_site_guards.get(&call_inst).map_or(&[], |v| v.as_slice())
    }
}

// In pathsens_runner.rs - augment finding guards with caller context
fn augment_with_caller_guards(
    finding: &CheckerFinding,
    caller_ctx: &CallerGuardContext,
    callgraph: &CallGraph,
) -> PathCondition {
    let mut guards = extract_guards(&finding.trace, &index);

    // Find call sites that lead to this finding's function
    // Add their guards to the path condition
    // ...

    guards
}
```

### Expected Impact
- 15-25% reduction in false positives from interprocedural analysis
- Functions called only from guarded contexts will not report false alarms

---

## Implementation Order

| Phase | Effort | Impact | Priority |
|-------|--------|--------|----------|
| Phase 1: Null-Check Sanitizers | ~200 LOC | HIGH (60-70% FP reduction for null-deref) | **P0** |
| Phase 2: Load/Store Classification | ~150 LOC | HIGH (enables proper dereference detection) | **P0** |
| Phase 3: MemPhi Guard Bridging | ~100 LOC | MEDIUM (10-20% FP reduction) | P1 |
| Phase 4: Constant Resolution | ~100 LOC | LOW (5-10% FP reduction) | P2 |
| Phase 5: Interprocedural Guards | ~200 LOC | MEDIUM (15-25% FP reduction) | P2 |

**Recommended order:** Phase 1 → Phase 2 → Phase 3 → Phase 4 → Phase 5

Total: ~750 LOC

---

## Verification

After implementation, run:
```bash
make test-svcomp-category CAT=memsafety
```

**Success criteria:**
- Incorrect FALSE verdicts: < 20 (currently 79)
- Correct FALSE verdicts: > 50 (currently 31)
- Score: > 0 (currently -1790)

---

## Alternative Approaches Considered

### A. Stronger PTA for Alias Precision
- Would help with may-alias false positives
- Higher effort (~500+ LOC), lower impact for null-deref specifically
- Defer to future work

### B. CEGAR Refinement Loop
- Already implemented in E31 (Plan 054)
- Could be integrated with checker framework
- Medium effort, needs counterexample generation

### C. Witness-Guided Validation
- Use SV-COMP witness format to validate findings
- Requires parsing expected witnesses
- Out of scope for precision improvements

---

## Dependencies

- SVFG graph (E12) ✓
- Path-sensitive filtering (E18) ✓
- Z3 solver integration (E19) ✓
- Checker framework (E14) ✓

All dependencies satisfied.
