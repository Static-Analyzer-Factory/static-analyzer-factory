# Plan 054: SV-COMP Library Enhancements

## Overview

Enhance saf-analysis library with features that would significantly improve SV-COMP benchmark performance. While Plan 053 covers analyzer-side improvements (fast paths, conservative verdicts), this plan focuses on **library-level capabilities** that enable more precise verdicts.

**Current state:** +2 points (1 TRUE, 135 UNKNOWN)

**Target state:** +50-100 points through better precision and witness generation

## Analysis: Why UNKNOWN?

| Property | Current Limitation | Library Gap |
|----------|-------------------|-------------|
| `unreach-call` | Z3 times out on loops | No loop invariant synthesis |
| `no-overflow` | Intervals widen to ⊤ in loops | No relational domains (octagon) |
| `valid-memsafety` | False positives not refined | No CEGAR refinement |
| `no-data-race` | Imprecise MHP | No lock-sensitivity |
| All FALSE verdicts | No witnesses | No GraphML export |

## Prioritized Enhancements

### Priority 1: Octagon Domain (High Impact)

**Problem:** Interval domain loses relational constraints like `i < n` in loops.

**Solution:** Implement octagon domain tracking `±x ± y ≤ c` constraints.

**Impact:**
- `no-overflow`: Prove bounds on loop counters relative to array lengths
- `unreach-call`: Prove guards like `if (i >= n) error()` unreachable

**Effort:** ~1-2 weeks

---

### Priority 2: GraphML Witness Generation (Required for FALSE)

**Problem:** SV-COMP requires GraphML violation/correctness witnesses for scoring.

**Solution:** Generate SV-COMP 2.0 witness format from analysis traces.

**Impact:**
- +1 point per correct FALSE verdict (currently 0)
- Enables verification by witness validators

**Effort:** ~3-5 days

---

### Priority 3: CEGAR Refinement Loop (Medium Impact)

**Problem:** Initial abstraction may be too coarse; false positives returned as UNKNOWN.

**Solution:** Counterexample-guided abstraction refinement:
1. Run analysis with coarse abstraction
2. If finding is potentially false positive, refine abstraction
3. Re-analyze with refined abstraction
4. Repeat until proven or budget exhausted

**Impact:**
- Reduce false positives that cause UNKNOWN verdicts
- Enable confident TRUE verdicts

**Effort:** ~1-2 weeks

---

### Priority 4: Loop Invariant Synthesis (High Impact, Complex)

**Problem:** Widening produces ⊤ quickly; can't prove loop properties.

**Solution:** Template-based invariant synthesis:
1. Generate candidate invariants from loop guards and assignments
2. Use abstract interpretation to check candidates
3. Strengthen candidates until fixpoint

**Impact:**
- Prove bounded loops terminate with values in range
- Enable precise `no-overflow` verdicts

**Effort:** ~2-3 weeks

---

### Priority 5: Lock-Sensitive MHP (Medium Impact)

**Problem:** MHP analysis doesn't account for synchronization.

**Solution:** Track lock-held sets per program point; accesses under same lock don't race.

**Impact:**
- Reduce false positive race reports
- Enable TRUE verdicts for properly synchronized programs

**Effort:** ~1 week

---

## Phase 1: Octagon Domain (~400 LOC)

### 1.1 DBM Representation

**File:** `crates/saf-analysis/src/absint/octagon/dbm.rs`

```rust
/// Difference-Bound Matrix for octagon constraints.
/// Represents constraints of the form: x_i - x_j <= c
///
/// For octagons, we use 2n variables: x_i^+ and x_i^- for each variable i,
/// where x_i^+ = x_i and x_i^- = -x_i.
///
/// This allows encoding ±x ± y <= c as difference constraints.
#[derive(Clone, Debug)]
pub struct Dbm {
    /// Number of program variables (not DBM variables)
    num_vars: usize,
    /// Matrix entries: m[i][j] represents x_i - x_j <= m[i][j]
    /// Size is 2n x 2n where n = num_vars
    matrix: Vec<Vec<Bound>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bound {
    /// Finite bound
    Finite(i128),
    /// No constraint (infinity)
    PosInf,
}

impl Dbm {
    /// Create empty DBM (all constraints are PosInf)
    pub fn top(num_vars: usize) -> Self;

    /// Create inconsistent DBM (bottom)
    pub fn bottom(num_vars: usize) -> Self;

    /// Add constraint: var1 - var2 <= bound
    pub fn add_constraint(&mut self, var1: VarIndex, var2: VarIndex, bound: i128);

    /// Add octagon constraint: coef1*x + coef2*y <= c
    /// where coef1, coef2 ∈ {-1, 0, +1}
    pub fn add_octagon(&mut self, x: VarIndex, coef_x: i8, y: VarIndex, coef_y: i8, c: i128);

    /// Close the DBM using Floyd-Warshall
    pub fn close(&mut self);

    /// Check if DBM is consistent (no negative cycle)
    pub fn is_consistent(&self) -> bool;

    /// Get interval for variable (from diagonal constraints)
    pub fn get_interval(&self, var: VarIndex) -> Interval;
}
```

### 1.2 Octagon Domain

**File:** `crates/saf-analysis/src/absint/octagon/domain.rs`

```rust
use super::dbm::Dbm;

/// Octagon abstract domain tracking constraints ±x ± y <= c.
///
/// More precise than intervals for relational properties but O(n^3) operations.
#[derive(Clone, Debug)]
pub struct OctagonDomain {
    /// Mapping from ValueId to DBM variable index
    var_map: BTreeMap<ValueId, VarIndex>,
    /// The difference-bound matrix
    dbm: Dbm,
}

impl AbstractDomain for OctagonDomain {
    fn top() -> Self;
    fn bottom() -> Self;
    fn is_bottom(&self) -> bool;
    fn join(&self, other: &Self) -> Self;  // Pointwise max
    fn meet(&self, other: &Self) -> Self;  // Pointwise min + closure
    fn widening(&self, other: &Self) -> Self;  // Standard octagon widening
    fn narrowing(&self, other: &Self) -> Self;
    fn leq(&self, other: &Self) -> bool;
}

impl OctagonDomain {
    /// Get interval bounds for a variable (projection)
    pub fn get_interval(&self, value: ValueId) -> Interval;

    /// Apply assignment: x := y + c
    pub fn assign_linear(&mut self, x: ValueId, y: ValueId, c: i128);

    /// Apply guard: x <= y + c
    pub fn assume_leq(&mut self, x: ValueId, y: ValueId, c: i128);

    /// Forget a variable (havoc)
    pub fn forget(&mut self, x: ValueId);
}
```

### 1.3 Octagon Transfer Functions

**File:** `crates/saf-analysis/src/absint/octagon/transfer.rs`

```rust
/// Transfer function for octagon domain.
pub fn octagon_transfer(
    inst: &Instruction,
    state: &mut OctagonDomain,
    module: &AirModule,
) {
    match &inst.operation {
        // x = y + c (constant addition)
        Operation::BinaryOp { op: BinaryOpKind::Add, left, right, .. } => {
            if let Some(c) = get_constant(right, module) {
                state.assign_linear(inst.result, *left, c);
            } else {
                // Non-linear: fall back to interval projection
                state.forget(inst.result);
            }
        }

        // Comparisons generate constraints
        Operation::ICmp { predicate: ICmpPredicate::Slt, left, right, .. } => {
            // if x < y, then x - y <= -1
            state.assume_leq(*left, *right, -1);
        }

        // ... other operations
    }
}
```

### 1.4 Integration with Fixpoint

**File:** `crates/saf-analysis/src/absint/octagon/mod.rs`

```rust
pub mod dbm;
pub mod domain;
pub mod transfer;

pub use domain::OctagonDomain;

/// Run octagon analysis on a function.
pub fn solve_octagon(
    func: &AirFunction,
    cfg: &Cfg,
    config: &OctagonConfig,
) -> OctagonResult;

/// Configuration for octagon analysis.
pub struct OctagonConfig {
    /// Maximum variables to track (octagon is O(n^3))
    pub max_vars: usize,
    /// Widening delay iterations
    pub widening_delay: usize,
}
```

---

## Phase 2: GraphML Witness Generation (~300 LOC)

### 2.1 Witness Types

**File:** `crates/saf-bench/src/svcomp/witness.rs`

```rust
/// SV-COMP witness format version 2.0
pub enum WitnessType {
    /// Violation witness (for FALSE verdicts)
    Violation,
    /// Correctness witness (for TRUE verdicts)
    Correctness,
}

/// A witness graph in SV-COMP GraphML format.
pub struct Witness {
    pub witness_type: WitnessType,
    pub producer: String,
    pub specification: String,
    pub program_file: String,
    pub program_hash: String,
    pub architecture: String,
    pub creation_time: String,
    pub nodes: Vec<WitnessNode>,
    pub edges: Vec<WitnessEdge>,
}

pub struct WitnessNode {
    pub id: String,
    pub entry: bool,
    pub sink: bool,
    pub violation: bool,
}

pub struct WitnessEdge {
    pub source: String,
    pub target: String,
    pub source_code: Option<String>,
    pub start_line: Option<u32>,
    pub assumption: Option<String>,
}
```

### 2.2 Witness Builder

**File:** `crates/saf-bench/src/svcomp/witness.rs` (continued)

```rust
impl Witness {
    /// Create violation witness from analysis trace.
    pub fn from_violation_trace(
        trace: &[BlockId],
        module: &AirModule,
        property: &str,
        program_path: &Path,
    ) -> Self;

    /// Create correctness witness (loop invariants).
    pub fn from_invariants(
        invariants: &BTreeMap<BlockId, String>,
        module: &AirModule,
        property: &str,
        program_path: &Path,
    ) -> Self;

    /// Export to GraphML string.
    pub fn to_graphml(&self) -> String;

    /// Write to file.
    pub fn write_to_file(&self, path: &Path) -> io::Result<()>;
}
```

### 2.3 GraphML Serialization

```rust
impl Witness {
    pub fn to_graphml(&self) -> String {
        let mut xml = String::new();
        xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        xml.push_str("\n");
        xml.push_str(r#"<graphml xmlns="http://graphml.graphdrawing.org/xmlns">"#);
        xml.push_str("\n");

        // Key declarations
        xml.push_str(r#"  <key id="witness-type" for="graph" attr.type="string"/>"#);
        xml.push_str("\n");
        // ... more keys

        // Graph with data
        xml.push_str(r#"  <graph edgedefault="directed">"#);
        xml.push_str("\n");
        xml.push_str(&format!(
            r#"    <data key="witness-type">{}</data>"#,
            self.witness_type.as_str()
        ));
        // ... nodes and edges

        xml.push_str("  </graph>\n");
        xml.push_str("</graphml>\n");
        xml
    }
}
```

### 2.4 Integration with Property Analyzers

**File:** `crates/saf-bench/src/svcomp/property.rs` (modification)

```rust
pub struct PropertyResult {
    pub verdict: PropertyVerdict,
    pub witness: Option<Witness>,  // Add witness field
}

fn analyze_unreachability(ctx: &AnalysisContext<'_>) -> PropertyResult {
    // ... existing analysis

    match result.result {
        PathReachability::Reachable(path) => {
            let witness = Witness::from_violation_trace(
                &path, ctx.module, "unreach-call", &ctx.config.program_path
            );
            PropertyResult {
                verdict: PropertyVerdict::False { reason: "Error reachable".into() },
                witness: Some(witness),
            }
        }
        // ...
    }
}
```

---

## Phase 3: CEGAR Refinement (~500 LOC)

### 3.1 Abstraction State

**File:** `crates/saf-analysis/src/cegar/abstraction.rs`

```rust
/// Abstraction level for CEGAR.
#[derive(Clone, Debug)]
pub struct Abstraction {
    /// Context sensitivity level for PTA
    pub pta_k: usize,
    /// Whether to use flow-sensitive PTA
    pub flow_sensitive: bool,
    /// Tracked predicates for path sensitivity
    pub predicates: BTreeSet<Predicate>,
    /// Variables to track precisely (others are havoc'd)
    pub tracked_vars: BTreeSet<ValueId>,
}

impl Abstraction {
    /// Initial coarse abstraction
    pub fn initial() -> Self {
        Self {
            pta_k: 0,  // Context-insensitive
            flow_sensitive: false,
            predicates: BTreeSet::new(),
            tracked_vars: BTreeSet::new(),
        }
    }

    /// Refine based on spurious counterexample
    pub fn refine(&self, counterexample: &Counterexample) -> Self;
}
```

### 3.2 Counterexample Analysis

**File:** `crates/saf-analysis/src/cegar/counterexample.rs`

```rust
/// A potential counterexample (may be spurious).
pub struct Counterexample {
    pub trace: Vec<InstId>,
    pub property_violation: PropertyViolation,
}

/// Result of checking a counterexample.
pub enum CexCheckResult {
    /// Counterexample is real
    Real,
    /// Counterexample is spurious; refinement hints provided
    Spurious { refinement: RefinementHint },
}

pub struct RefinementHint {
    /// Predicates to add
    pub new_predicates: Vec<Predicate>,
    /// Variables needing precise tracking
    pub track_vars: Vec<ValueId>,
    /// Increase context sensitivity
    pub increase_k: bool,
}

/// Check if counterexample is feasible using Z3.
pub fn check_counterexample(
    cex: &Counterexample,
    module: &AirModule,
    timeout_ms: u64,
) -> CexCheckResult;
```

### 3.3 CEGAR Loop

**File:** `crates/saf-analysis/src/cegar/mod.rs`

```rust
pub mod abstraction;
pub mod counterexample;

pub use abstraction::Abstraction;
pub use counterexample::{Counterexample, CexCheckResult};

/// CEGAR configuration.
pub struct CegarConfig {
    /// Maximum refinement iterations
    pub max_iterations: usize,
    /// Z3 timeout per counterexample check
    pub z3_timeout_ms: u64,
}

/// Run CEGAR loop for property verification.
pub fn verify_with_cegar<A: Analysis>(
    analysis: A,
    module: &AirModule,
    property: &PropertySpec,
    config: &CegarConfig,
) -> CegarResult {
    let mut abstraction = Abstraction::initial();

    for iteration in 0..config.max_iterations {
        // Run analysis with current abstraction
        let result = analysis.run(module, &abstraction);

        match result {
            AnalysisResult::Safe => {
                return CegarResult::Verified { iterations: iteration + 1 };
            }
            AnalysisResult::PotentialViolation(cex) => {
                // Check if counterexample is real
                match check_counterexample(&cex, module, config.z3_timeout_ms) {
                    CexCheckResult::Real => {
                        return CegarResult::Violated { counterexample: cex };
                    }
                    CexCheckResult::Spurious { refinement } => {
                        // Refine abstraction and continue
                        abstraction = abstraction.refine_with_hint(&refinement);
                    }
                }
            }
        }
    }

    CegarResult::Unknown { reason: "Max iterations reached".into() }
}
```

---

## Phase 4: Loop Invariant Synthesis (~400 LOC)

### 4.1 Invariant Templates

**File:** `crates/saf-analysis/src/invariants/templates.rs`

```rust
/// Template for loop invariants.
#[derive(Clone, Debug)]
pub enum InvariantTemplate {
    /// x in [lo, hi]
    Interval { var: ValueId, lo: Option<i128>, hi: Option<i128> },
    /// x <= y + c
    LinearLeq { left: ValueId, right: ValueId, constant: i128 },
    /// x == y + c
    LinearEq { left: ValueId, right: ValueId, constant: i128 },
    /// x mod m == r
    Modular { var: ValueId, modulus: i128, remainder: i128 },
}

/// Generate candidate templates from loop structure.
pub fn generate_templates(
    loop_header: BlockId,
    loop_body: &[BlockId],
    module: &AirModule,
) -> Vec<InvariantTemplate>;
```

### 4.2 Invariant Checking

**File:** `crates/saf-analysis/src/invariants/checker.rs`

```rust
/// Check if invariant holds at loop header.
pub fn check_invariant(
    template: &InvariantTemplate,
    loop_header: BlockId,
    cfg: &Cfg,
    module: &AirModule,
    initial_state: &AbstractState,
) -> InvariantCheckResult;

pub enum InvariantCheckResult {
    /// Invariant holds (inductive)
    Valid,
    /// Invariant doesn't hold; counterexample provided
    Invalid { counterexample: Vec<InstId> },
    /// Couldn't determine
    Unknown,
}
```

### 4.3 Synthesis Algorithm

**File:** `crates/saf-analysis/src/invariants/mod.rs`

```rust
pub mod templates;
pub mod checker;

/// Synthesize loop invariants for a function.
pub fn synthesize_invariants(
    func: &AirFunction,
    cfg: &Cfg,
    config: &InvariantConfig,
) -> BTreeMap<BlockId, Vec<InvariantTemplate>> {
    let mut invariants = BTreeMap::new();

    // Find all loops (back edges)
    for loop_header in cfg.loop_headers() {
        let loop_body = cfg.loop_body(loop_header);

        // Generate candidate templates
        let candidates = generate_templates(loop_header, &loop_body, func.module);

        // Check each candidate
        let valid = candidates.into_iter()
            .filter(|t| check_invariant(t, loop_header, cfg, func.module, &initial)
                        == InvariantCheckResult::Valid)
            .collect();

        invariants.insert(loop_header, valid);
    }

    invariants
}
```

---

## Phase 5: Lock-Sensitive MHP (~300 LOC)

### 5.1 Lock Set Tracking

**File:** `crates/saf-analysis/src/mta/lockset.rs`

```rust
/// Set of locks held at a program point.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LockSet {
    locks: BTreeSet<LockId>,
}

impl LockSet {
    pub fn empty() -> Self { Self::default() }
    pub fn acquire(&mut self, lock: LockId);
    pub fn release(&mut self, lock: LockId);
    pub fn holds(&self, lock: LockId) -> bool;
    pub fn intersects(&self, other: &Self) -> bool;
}

/// Compute lock sets at each program point.
pub fn compute_locksets(
    module: &AirModule,
    cfg: &Cfg,
) -> BTreeMap<InstId, LockSet>;
```

### 5.2 Lock-Aware Race Detection

**File:** `crates/saf-analysis/src/mta/race.rs` (modification)

```rust
/// Check for data race with lock sensitivity.
pub fn check_race_with_locks(
    access1: &MemoryAccess,
    access2: &MemoryAccess,
    locksets: &BTreeMap<InstId, LockSet>,
    mhp: &MhpResult,
    pta: &PtaResult,
) -> RaceCheckResult {
    // Not MHP → no race
    if !mhp.may_happen_in_parallel(access1.thread, access2.thread) {
        return RaceCheckResult::NoRace;
    }

    // Different memory → no race
    if pta.may_alias(access1.location, access2.location) == AliasResult::No {
        return RaceCheckResult::NoRace;
    }

    // Both reads → no race
    if access1.is_read && access2.is_read {
        return RaceCheckResult::NoRace;
    }

    // Check if protected by common lock
    let locks1 = locksets.get(&access1.inst).unwrap_or(&LockSet::empty());
    let locks2 = locksets.get(&access2.inst).unwrap_or(&LockSet::empty());

    if locks1.intersects(locks2) {
        return RaceCheckResult::NoRace;  // Protected by common lock
    }

    RaceCheckResult::PotentialRace
}
```

---

## Files to Create/Modify

### New Files

| File | Lines | Description |
|------|-------|-------------|
| `absint/octagon/mod.rs` | ~50 | Octagon module exports |
| `absint/octagon/dbm.rs` | ~200 | DBM representation |
| `absint/octagon/domain.rs` | ~150 | Octagon abstract domain |
| `absint/octagon/transfer.rs` | ~100 | Transfer functions |
| `cegar/mod.rs` | ~100 | CEGAR loop |
| `cegar/abstraction.rs` | ~150 | Abstraction state |
| `cegar/counterexample.rs` | ~150 | Counterexample checking |
| `invariants/mod.rs` | ~100 | Invariant synthesis |
| `invariants/templates.rs` | ~150 | Template generation |
| `invariants/checker.rs` | ~150 | Invariant checking |
| `mta/lockset.rs` | ~100 | Lock set tracking |
| `svcomp/witness.rs` | ~300 | GraphML witness generation |

**Total new: ~1700 LOC**

### Modified Files

| File | Changes |
|------|---------|
| `absint/mod.rs` | Re-export octagon module |
| `mta/race.rs` | Add lock-sensitive race checking |
| `mta/mod.rs` | Re-export lockset module |
| `lib.rs` | Re-export cegar, invariants |
| `svcomp/property.rs` | Use octagon, CEGAR, witnesses |
| `svcomp/mod.rs` | Re-export witness module |

---

## Implementation Order

| Phase | Deliverable | Effort | SV-COMP Impact |
|-------|-------------|--------|----------------|
| 1 | Octagon domain | 1-2 weeks | +10-20 TRUE (no-overflow) |
| 2 | GraphML witnesses | 3-5 days | Enables FALSE scoring |
| 3 | CEGAR refinement | 1-2 weeks | +10-20 TRUE (reduce FP) |
| 4 | Loop invariants | 2-3 weeks | +20-30 TRUE (loops) |
| 5 | Lock-sensitive MHP | 1 week | +5-10 TRUE (no-data-race) |

**Recommended order:** 2 → 1 → 3 → 5 → 4

Rationale:
- Phase 2 (witnesses) enables FALSE scoring immediately
- Phase 1 (octagon) improves precision for overflow checks
- Phase 3 (CEGAR) reduces false positives systematically
- Phase 5 (locks) is relatively easy win for race detection
- Phase 4 (invariants) is highest complexity, tackle last

---

## Testing Strategy

### Unit Tests

- DBM closure correctness (Floyd-Warshall)
- Octagon join/meet/widening properties
- GraphML well-formedness
- CEGAR iteration termination
- Invariant template generation
- Lock set dataflow

### Integration Tests

1. **Octagon E2E:** Loop with relational guard (`while (i < n)`) proves no overflow
2. **Witness E2E:** FALSE verdict generates valid GraphML
3. **CEGAR E2E:** Spurious counterexample eliminated by refinement
4. **Invariant E2E:** Simple loop invariant synthesized and verified
5. **Lock E2E:** Properly synchronized accesses proven race-free

### SV-COMP Validation

```bash
# Run with enhanced library
saf-bench svcomp --compiled-dir .compiled --json > results.json

# Check for improvement
jq '.summary.true_correct' results.json  # Target: 50+
jq '.summary.incorrect' results.json     # Must be: 0
```

---

## Success Criteria

| Metric | Current | Target |
|--------|---------|--------|
| TRUE correct | 1 | 50+ |
| FALSE correct | 0 | 10+ |
| Incorrect verdicts | 0 | 0 |
| UNKNOWN | 135 | <100 |
| Score | +2 | +50-100 |

---

## Dependencies

- **Octagon:** None (builds on existing absint infrastructure)
- **Witnesses:** Plan 053 (analyzer improvements for trace generation)
- **CEGAR:** Z3 (already integrated via E19)
- **Invariants:** Octagon domain (Phase 1)
- **Lock MHP:** Existing MTA module

---

## Future Extensions

After this plan:

1. **Polyhedra domain** — Full linear constraints (requires APRON/ELINA FFI)
2. **Termination analysis** — Ranking functions for `termination` property
3. **Correctness witnesses** — Loop invariants as correctness certificates
4. **Parallel CEGAR** — Multi-core refinement exploration
