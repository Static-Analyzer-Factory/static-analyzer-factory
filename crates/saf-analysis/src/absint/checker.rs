//! Numeric checkers for buffer overflow (CWE-120) and integer overflow (CWE-190).
//!
//! These checkers query the abstract interpretation invariants at specific
//! program points to detect numeric bugs.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use saf_core::air::{
    AirBlock, AirFunction, AirModule, AirType, BinaryOp, Constant, Instruction, Operation,
};
use saf_core::ids::{BlockId, FunctionId, LocId, ObjId, TypeId, ValueId};
use saf_core::spec::{AnalyzedSpecRegistry, SpecRegistry};

use super::pta_integration::PtaIntegration;

/// Default value for the location field (used by serde skip).
fn default_location() -> (FunctionId, BlockId) {
    (FunctionId::new(0), BlockId::new(0))
}

use super::config::AbstractInterpConfig;
use super::domain::AbstractDomain;
use super::fixpoint::{solve_abstract_interp, solve_abstract_interp_with_specs};
use super::interval::{Interval, signed_max, signed_min};
use super::result::AbstractInterpResult;

/// Severity level for a numeric finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NumericSeverity {
    /// The operation is provably safe.
    Safe,
    /// The operation may be unsafe (cannot prove safe or definitely unsafe).
    Warning,
    /// The operation is provably unsafe (definite bug or dead code).
    Error,
}

impl NumericSeverity {
    /// Get the name as a string.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Safe => "safe",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
}

/// Kind of numeric checker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NumericCheckerKind {
    /// Buffer overflow (CWE-120): array index out of bounds.
    BufferOverflow,
    /// Integer overflow (CWE-190): arithmetic result exceeds bit-width.
    IntegerOverflow,
    /// Division by zero (CWE-369): divisor may be zero.
    DivisionByZero,
    /// Invalid shift count (CWE-682): shift amount is negative or >= bit-width.
    ShiftCount,
}

impl NumericCheckerKind {
    /// Get the CWE ID for this checker.
    #[must_use]
    pub fn cwe(&self) -> u32 {
        match self {
            Self::BufferOverflow => 120,
            Self::IntegerOverflow => 190,
            Self::DivisionByZero => 369,
            Self::ShiftCount => 682,
        }
    }

    /// Get the name as a string.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::BufferOverflow => "buffer_overflow",
            Self::IntegerOverflow => "integer_overflow",
            Self::DivisionByZero => "division_by_zero",
            Self::ShiftCount => "shift_count",
        }
    }
}

/// A finding from a numeric checker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumericFinding {
    /// The checker that produced this finding.
    pub checker: NumericCheckerKind,
    /// Severity level.
    pub severity: NumericSeverity,
    /// CWE ID.
    pub cwe: u32,
    /// Instruction ID where the finding was detected.
    pub inst_id: String,
    /// Human-readable description.
    pub description: String,
    /// The computed interval (as string).
    pub interval: String,
    /// Function name where the finding occurs.
    pub function: String,
    /// Location of the finding: `(FunctionId, BlockId)`.
    #[serde(skip, default = "default_location")]
    pub location: (saf_core::ids::FunctionId, saf_core::ids::BlockId),
    /// The affected pointer/buffer (for matching with oracles).
    /// This is the destination pointer for memcpy/GEP overflow checks.
    #[serde(skip, default)]
    pub affected_ptr: Option<ValueId>,
}

impl Default for NumericFinding {
    fn default() -> Self {
        Self {
            checker: NumericCheckerKind::BufferOverflow,
            severity: NumericSeverity::Warning,
            cwe: 0,
            inst_id: String::new(),
            description: String::new(),
            interval: String::new(),
            function: String::new(),
            location: default_location(),
            affected_ptr: None,
        }
    }
}

/// Result of running a numeric check.
#[derive(Debug, Clone)]
pub struct NumericCheckResult {
    /// All findings from the check.
    pub findings: Vec<NumericFinding>,
    /// The abstract interpretation result (for further queries).
    pub absint_result: Option<AbstractInterpResult>,
}

/// Check for buffer overflow (CWE-120).
///
/// At each GEP with a dynamic index operand, checks whether the index
/// interval is within `[0, allocation_size)`. Since we don't have full
/// allocation tracking (no memory abstract domain), we check for:
/// - Negative indices (always bad)
/// - Indices that might be very large (suspicious)
///
/// For HeapAlloc sites with known size arguments, we track the size.
#[must_use]
pub fn check_buffer_overflow(
    module: &AirModule,
    config: &AbstractInterpConfig,
) -> NumericCheckResult {
    let result = solve_abstract_interp(module, config);
    let mut findings = Vec::new();

    // Track allocation sizes: InstId → Interval (size)
    let alloca_size_map = build_alloca_size_map(module);
    let alloc_sizes = extract_allocation_sizes(module, &result, &alloca_size_map);

    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                // Check GEP with dynamic index
                if let Operation::Gep { field_path: _ } = &inst.op {
                    // The last operand of GEP is the index
                    if inst.operands.len() >= 2 {
                        let idx_operand = inst.operands[inst.operands.len() - 1];
                        let state = result.state_at_inst(inst.id);
                        let idx_interval = state
                            .and_then(|s| s.get_opt(idx_operand).cloned())
                            .unwrap_or_else(|| Interval::make_top(64));

                        if idx_interval.is_bottom() {
                            continue; // Unreachable code
                        }

                        // Check 1: negative index
                        if idx_interval.lo() < 0 {
                            let severity = if idx_interval.hi() < 0 {
                                NumericSeverity::Error // Always negative
                            } else {
                                NumericSeverity::Warning // May be negative
                            };

                            findings.push(NumericFinding {
                                checker: NumericCheckerKind::BufferOverflow,
                                severity,
                                cwe: 120,
                                inst_id: inst.id.to_hex(),
                                description: format!("Array index may be negative: {idx_interval}"),
                                interval: format!("{idx_interval:?}"),
                                function: func.name.clone(),
                                location: (func.id, block.id),
                                affected_ptr: None,
                            });
                        }

                        // Check 2: index vs known allocation size
                        let base_operand = inst.operands[0];
                        if let Some(alloc_size) = alloc_sizes.get(&base_operand) {
                            if !alloc_size.is_top() && !alloc_size.is_bottom() {
                                let size_hi = alloc_size.hi();
                                if size_hi > 0 && idx_interval.hi() >= size_hi {
                                    let severity = if idx_interval.lo() >= size_hi {
                                        NumericSeverity::Error
                                    } else {
                                        NumericSeverity::Warning
                                    };
                                    findings.push(NumericFinding {
                                        checker: NumericCheckerKind::BufferOverflow,
                                        severity,
                                        cwe: 120,
                                        inst_id: inst.id.to_hex(),
                                        description: format!(
                                            "Array index {idx_interval} may exceed allocation size {alloc_size}"
                                        ),
                                        interval: format!("{idx_interval:?}"),
                                        function: func.name.clone(),
                                        location: (func.id, block.id),
                                affected_ptr: None,
                                    });
                                }
                            }
                        }

                        // Check 3: very large index without known size (heuristic)
                        if idx_interval.is_top()
                            || (idx_interval.hi() > 1_000_000
                                && !alloc_sizes.contains_key(&base_operand))
                        {
                            // Only warn if we have no size info and index is unbounded
                            if !alloc_sizes.contains_key(&base_operand) && !idx_interval.is_top() {
                                findings.push(NumericFinding {
                                    checker: NumericCheckerKind::BufferOverflow,
                                    severity: NumericSeverity::Warning,
                                    cwe: 120,
                                    inst_id: inst.id.to_hex(),
                                    description: format!(
                                        "Array index {idx_interval} is very large without known bounds"
                                    ),
                                    interval: format!("{idx_interval:?}"),
                                    function: func.name.clone(),
                                    location: (func.id, block.id),
                                affected_ptr: None,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    NumericCheckResult {
        findings,
        absint_result: Some(result),
    }
}

/// Check for buffer overflow (CWE-120) with spec-aware abstract interpretation.
///
/// Like [`check_buffer_overflow`], but uses `solve_abstract_interp_with_specs`
/// to apply return interval specs for external functions (e.g., `atoi` returns
/// `[INT_MIN, INT_MAX]` instead of TOP). This allows the checker to detect
/// overflows involving external input sources.
#[must_use]
pub fn check_buffer_overflow_with_specs(
    module: &AirModule,
    config: &AbstractInterpConfig,
    specs: Option<&AnalyzedSpecRegistry>,
) -> NumericCheckResult {
    let result = solve_abstract_interp_with_specs(module, config, specs);
    let findings = check_buffer_overflow_with_result(module, &result);
    NumericCheckResult {
        findings,
        absint_result: Some(result),
    }
}

/// Check for buffer overflow with PTA integration (CWE-120).
///
/// Extends `check_buffer_overflow` with PTA for:
/// - Resolving GEP base pointers to allocation sites via alias analysis
/// - Better pointer origin tracking for memcpy checks
///
/// Uses the points-to set to find allocation sites for GEP base pointers,
/// improving precision when the base pointer is derived from an allocation.
#[must_use]
pub fn check_buffer_overflow_with_pta(
    module: &AirModule,
    config: &AbstractInterpConfig,
    pta: &PtaIntegration<'_>,
) -> NumericCheckResult {
    let result = solve_abstract_interp(module, config);
    let findings = check_buffer_overflow_with_pta_and_result(module, &result, pta);
    NumericCheckResult {
        findings,
        absint_result: Some(result),
    }
}

/// Check for integer overflow (CWE-190).
///
/// At each arithmetic operation (Add, Sub, Mul), computes the mathematical
/// result interval without clamping and checks whether it fits within the
/// target bit-width range.
#[must_use]
pub fn check_integer_overflow(
    module: &AirModule,
    config: &AbstractInterpConfig,
) -> NumericCheckResult {
    let result = solve_abstract_interp(module, config);
    let mut findings = Vec::new();

    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                if let Operation::BinaryOp { kind } = &inst.op {
                    // Only check integer arithmetic (not comparisons, floats, or bitwise)
                    if !matches!(kind, BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul) {
                        continue;
                    }

                    if inst.operands.len() < 2 {
                        continue;
                    }

                    let state = result.state_at_inst(inst.id);
                    let lhs = state
                        .and_then(|s| s.get_opt(inst.operands[0]).cloned())
                        .unwrap_or_else(|| Interval::make_top(64));
                    let rhs = state
                        .and_then(|s| s.get_opt(inst.operands[1]).cloned())
                        .unwrap_or_else(|| Interval::make_top(64));

                    if lhs.is_bottom() || rhs.is_bottom() {
                        continue; // Unreachable
                    }
                    if lhs.is_top() || rhs.is_top() {
                        continue; // Can't check without bounds
                    }

                    // Compute unwrapped mathematical result
                    let (result_lo, result_hi) = match kind {
                        BinaryOp::Add => lhs.add_unwrapped(&rhs),
                        BinaryOp::Sub => lhs.sub_unwrapped(&rhs),
                        BinaryOp::Mul => lhs.mul_unwrapped(&rhs),
                        _ => continue,
                    };

                    // Check against the bit-width range
                    // Use the LHS bits as target (both operands should have same bits)
                    let bits = lhs.bits();
                    let target_min = signed_min(bits);
                    let target_max = signed_max(bits);

                    let fits_lo = result_lo >= target_min;
                    let fits_hi = result_hi <= target_max;

                    if fits_lo && fits_hi {
                        // Safe: result always fits
                        continue;
                    }

                    let severity = if !fits_lo && !fits_hi {
                        // Result always overflows in both directions
                        NumericSeverity::Error
                    } else if result_lo > target_max || result_hi < target_min {
                        // Result entirely outside target range
                        NumericSeverity::Error
                    } else {
                        NumericSeverity::Warning
                    };

                    let op_name = match kind {
                        BinaryOp::Add => "addition",
                        BinaryOp::Sub => "subtraction",
                        BinaryOp::Mul => "multiplication",
                        _ => "arithmetic",
                    };

                    findings.push(NumericFinding {
                        checker: NumericCheckerKind::IntegerOverflow,
                        severity,
                        cwe: 190,
                        inst_id: inst.id.to_hex(),
                        description: format!(
                            "Integer {op_name} may overflow i{bits}: [{result_lo}, {result_hi}] vs [{target_min}, {target_max}]"
                        ),
                        interval: format!("[{result_lo}, {result_hi}]"),
                        function: func.name.clone(),
                        location: (func.id, block.id),
                                affected_ptr: None,
                    });
                }
            }
        }
    }

    NumericCheckResult {
        findings,
        absint_result: Some(result),
    }
}

/// Check for integer overflow (CWE-190) with spec-aware abstract interpretation.
///
/// Like [`check_integer_overflow`], but uses `solve_abstract_interp_with_specs`
/// to apply return interval specs for external functions (e.g., `atoi` returns
/// `[INT_MIN, INT_MAX]` instead of TOP). This allows the checker to detect
/// overflows involving external input sources.
#[must_use]
pub fn check_integer_overflow_with_specs(
    module: &AirModule,
    config: &AbstractInterpConfig,
    specs: Option<&AnalyzedSpecRegistry>,
) -> NumericCheckResult {
    let result = solve_abstract_interp_with_specs(module, config, specs);
    let findings = check_integer_overflow_with_result(module, &result);
    NumericCheckResult {
        findings,
        absint_result: Some(result),
    }
}

/// Check for division by zero (CWE-369).
///
/// At each division or remainder operation (`SDiv`, `UDiv`, `SRem`, `URem`),
/// checks whether the divisor interval contains zero.
#[must_use]
pub fn check_division_by_zero(
    module: &AirModule,
    config: &AbstractInterpConfig,
) -> NumericCheckResult {
    let result = solve_abstract_interp(module, config);
    let findings = check_division_by_zero_with_result(module, &result);
    NumericCheckResult {
        findings,
        absint_result: Some(result),
    }
}

/// Check for invalid shift counts (CWE-682).
///
/// At each shift operation (`Shl`, `LShr`, `AShr`), checks whether the shift
/// amount is negative or >= the bit-width of the value being shifted.
#[must_use]
pub fn check_shift_count(module: &AirModule, config: &AbstractInterpConfig) -> NumericCheckResult {
    let result = solve_abstract_interp(module, config);
    let findings = check_shift_count_with_result(module, &result);
    NumericCheckResult {
        findings,
        absint_result: Some(result),
    }
}

/// Run all numeric checkers.
#[must_use]
pub fn check_all_numeric(module: &AirModule, config: &AbstractInterpConfig) -> NumericCheckResult {
    // Run abstract interpretation once and reuse
    let result = solve_abstract_interp(module, config);
    let mut all_findings = Vec::new();

    // Buffer overflow check
    let bo_findings = check_buffer_overflow_with_result(module, &result);
    all_findings.extend(bo_findings);

    // Integer overflow check
    let io_findings = check_integer_overflow_with_result(module, &result);
    all_findings.extend(io_findings);

    // Division by zero check
    let dbz_findings = check_division_by_zero_with_result(module, &result);
    all_findings.extend(dbz_findings);

    // Shift count check
    let sc_findings = check_shift_count_with_result(module, &result);
    all_findings.extend(sc_findings);

    // Memcpy/memmove/memset overflow check (no specs in this context)
    let mc_findings = check_memcpy_overflow_impl(module, &result, None);
    all_findings.extend(mc_findings);

    NumericCheckResult {
        findings: all_findings,
        absint_result: Some(result),
    }
}

/// Buffer overflow check using pre-computed abstract interpretation result.
fn check_buffer_overflow_with_result(
    module: &AirModule,
    result: &AbstractInterpResult,
) -> Vec<NumericFinding> {
    let mut findings = Vec::new();
    let alloca_size_map = build_alloca_size_map(module);
    let alloc_sizes = extract_allocation_sizes(module, result, &alloca_size_map);

    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                if let Operation::Gep { .. } = &inst.op {
                    if inst.operands.len() >= 2 {
                        let idx_operand = inst.operands[inst.operands.len() - 1];
                        let state = result.state_at_inst(inst.id);
                        let idx_interval = state
                            .and_then(|s| s.get_opt(idx_operand).cloned())
                            .unwrap_or_else(|| Interval::make_top(64));

                        if idx_interval.is_bottom() {
                            continue;
                        }

                        let base_operand = inst.operands[0];

                        if idx_interval.is_top() {
                            // Heuristic: unconstrained index into a known-size small buffer
                            // is a potential overflow. Report Warning since we can't prove
                            // it always overflows.
                            if let Some(asize) = alloc_sizes.get(&base_operand) {
                                if !asize.is_top()
                                    && !asize.is_bottom()
                                    && asize.hi() > 0
                                    && asize.hi() <= 4096
                                {
                                    findings.push(NumericFinding {
                                        checker: NumericCheckerKind::BufferOverflow,
                                        severity: NumericSeverity::Warning,
                                        cwe: 120,
                                        inst_id: inst.id.to_hex(),
                                        description: format!(
                                            "Unconstrained array index into buffer of size {asize}"
                                        ),
                                        interval: "TOP".to_string(),
                                        function: func.name.clone(),
                                        location: (func.id, block.id),
                                        affected_ptr: None,
                                    });
                                }
                            }
                            continue;
                        }

                        if idx_interval.lo() < 0 {
                            let severity = if idx_interval.hi() < 0 {
                                NumericSeverity::Error
                            } else {
                                NumericSeverity::Warning
                            };
                            findings.push(NumericFinding {
                                checker: NumericCheckerKind::BufferOverflow,
                                severity,
                                cwe: 120,
                                inst_id: inst.id.to_hex(),
                                description: format!("Array index may be negative: {idx_interval}"),
                                interval: format!("{idx_interval:?}"),
                                function: func.name.clone(),
                                location: (func.id, block.id),
                                affected_ptr: None,
                            });
                        }

                        if let Some(alloc_size) = alloc_sizes.get(&base_operand) {
                            if !alloc_size.is_top() && !alloc_size.is_bottom() {
                                let size_hi = alloc_size.hi();
                                if size_hi > 0 && idx_interval.hi() >= size_hi {
                                    let severity = if idx_interval.lo() >= size_hi {
                                        NumericSeverity::Error
                                    } else {
                                        NumericSeverity::Warning
                                    };
                                    findings.push(NumericFinding {
                                        checker: NumericCheckerKind::BufferOverflow,
                                        severity,
                                        cwe: 120,
                                        inst_id: inst.id.to_hex(),
                                        description: format!(
                                            "Array index {idx_interval} may exceed allocation size {alloc_size}"
                                        ),
                                        interval: format!("{idx_interval:?}"),
                                        function: func.name.clone(),
                                        location: (func.id, block.id),
                                affected_ptr: None,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    findings
}

/// Buffer overflow check with PTA integration using pre-computed result.
///
/// Uses PTA to resolve GEP base pointers to allocation sites.
// NOTE: This function checks GEP-based overflows with 3 checks per instruction
// (TOP index, negative index, size exceeded). Splitting would obscure the
// sequential check structure.
#[allow(clippy::too_many_lines)]
fn check_buffer_overflow_with_pta_and_result(
    module: &AirModule,
    result: &AbstractInterpResult,
    pta: &PtaIntegration<'_>,
) -> Vec<NumericFinding> {
    use super::transfer::build_obj_type_map;

    let mut findings = Vec::new();
    let alloca_size_map = build_alloca_size_map(module);
    let alloc_sizes = extract_allocation_sizes(module, result, &alloca_size_map);

    // Build a map from LocId → allocation size
    let loc_alloc_sizes = build_loc_allocation_sizes(module, result, pta);

    // Build field-level allocation sizes for struct-aware overflow detection
    let obj_type_map = build_obj_type_map(module);
    let field_alloc_sizes = build_field_allocation_sizes(module, pta, &obj_type_map);

    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                if let Operation::Gep { .. } = &inst.op {
                    if inst.operands.len() >= 2 {
                        let idx_operand = inst.operands[inst.operands.len() - 1];
                        let state = result.state_at_inst(inst.id);
                        let idx_interval = state
                            .and_then(|s| s.get_opt(idx_operand).cloned())
                            .unwrap_or_else(|| Interval::make_top(64));

                        if idx_interval.is_bottom() {
                            continue;
                        }

                        let base_operand = inst.operands[0];

                        if idx_interval.is_top() {
                            // Heuristic: unconstrained index into a known-size small buffer
                            // is a potential overflow. Report Warning since we can't prove
                            // it always overflows.
                            let alloc_size = find_allocation_size_with_pta(
                                base_operand,
                                &alloc_sizes,
                                &loc_alloc_sizes,
                                pta,
                            );
                            if let Some(ref asize) = alloc_size {
                                if !asize.is_top()
                                    && !asize.is_bottom()
                                    && asize.hi() > 0
                                    && asize.hi() <= 4096
                                {
                                    findings.push(NumericFinding {
                                        checker: NumericCheckerKind::BufferOverflow,
                                        severity: NumericSeverity::Warning,
                                        cwe: 120,
                                        inst_id: inst.id.to_hex(),
                                        description: format!(
                                            "Unconstrained array index into buffer of size {asize}"
                                        ),
                                        interval: "TOP".to_string(),
                                        function: func.name.clone(),
                                        location: (func.id, block.id),
                                        affected_ptr: None,
                                    });
                                }
                            }
                            continue;
                        }

                        // Check 1: negative index
                        if idx_interval.lo() < 0 {
                            let severity = if idx_interval.hi() < 0 {
                                NumericSeverity::Error
                            } else {
                                NumericSeverity::Warning
                            };
                            findings.push(NumericFinding {
                                checker: NumericCheckerKind::BufferOverflow,
                                severity,
                                cwe: 120,
                                inst_id: inst.id.to_hex(),
                                description: format!("Array index may be negative: {idx_interval}"),
                                interval: format!("{idx_interval:?}"),
                                function: func.name.clone(),
                                location: (func.id, block.id),
                                affected_ptr: None,
                            });
                        }

                        // Check 2: index vs known allocation size
                        // First try PTA-based resolution
                        let mut alloc_size = find_allocation_size_with_pta(
                            base_operand,
                            &alloc_sizes,
                            &loc_alloc_sizes,
                            pta,
                        );

                        // Field-aware refinement: if the base pointer has field GEP
                        // targets, use the field-level remaining size
                        if let Some(st) = state {
                            if let Some(field_targets) = st.resolve_field_gep(base_operand) {
                                for &(loc, offset) in field_targets {
                                    if let Some(field_size) = field_alloc_sizes.get(&(loc, offset))
                                    {
                                        alloc_size = Some(field_size.clone());
                                        break;
                                    }
                                }
                            }
                        }

                        if let Some(alloc_size) = alloc_size {
                            if !alloc_size.is_top() && !alloc_size.is_bottom() {
                                let size_hi = alloc_size.hi();
                                if size_hi > 0 && idx_interval.hi() >= size_hi {
                                    let severity = if idx_interval.lo() >= size_hi {
                                        NumericSeverity::Error
                                    } else {
                                        NumericSeverity::Warning
                                    };
                                    findings.push(NumericFinding {
                                        checker: NumericCheckerKind::BufferOverflow,
                                        severity,
                                        cwe: 120,
                                        inst_id: inst.id.to_hex(),
                                        description: format!(
                                            "Array index {idx_interval} may exceed allocation size {alloc_size}"
                                        ),
                                        interval: format!("{idx_interval:?}"),
                                        function: func.name.clone(),
                                        location: (func.id, block.id),
                                        affected_ptr: None,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    findings
}

/// Build a map from `LocId` to allocation size using PTA.
///
/// When CI-PTA conflates pointers from different functions (e.g., `_bad()` and
/// `_good()` in CWE test cases) to the same `LocId`, multiple allocations of
/// different sizes may map to the same location. We use `join` (interval union)
/// instead of overwriting so the resulting interval conservatively covers all
/// possible allocation sizes. This prevents false positives where a smaller
/// allocation from one function masks a larger (sufficient) allocation from
/// another function.
// `pta` (points-to analysis) and `pts` (points-to set) are distinct concepts
#[allow(clippy::similar_names)]
fn build_loc_allocation_sizes(
    module: &AirModule,
    result: &AbstractInterpResult,
    pta: &PtaIntegration<'_>,
) -> BTreeMap<LocId, Interval> {
    use super::transfer::build_constant_map;

    let mut sizes = BTreeMap::new();
    let constant_map = build_constant_map(module);

    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                // Track HeapAlloc allocations
                if let Operation::HeapAlloc { .. } = &inst.op {
                    if let Some(dst) = inst.dst {
                        if let Some(&size_operand) = inst.operands.first() {
                            let state = result.state_at_inst(inst.id);
                            let size_interval = state
                                .and_then(|s| s.get_opt(size_operand).cloned())
                                .or_else(|| constant_map.get(&size_operand).cloned())
                                .unwrap_or_else(|| Interval::make_top(64));

                            // Get LocIds that this allocation creates.
                            // Join with existing entry to handle CI-PTA conflation:
                            // when multiple allocations map to the same LocId, the
                            // joined interval covers all possible sizes.
                            let pts = pta.points_to(dst);
                            for loc in pts {
                                sizes
                                    .entry(loc)
                                    .and_modify(|existing: &mut Interval| {
                                        *existing = existing.join(&size_interval);
                                    })
                                    .or_insert_with(|| size_interval.clone());
                            }
                        }
                    }
                }

                // Track Alloca allocations — size_bytes is set by the frontend
                // for address-taken allocas that survive mem2reg.
                if let Operation::Alloca { size_bytes } = &inst.op {
                    if let Some(dst) = inst.dst {
                        let size_interval = if let Some(size) = size_bytes {
                            Interval::singleton(i128::from(*size), 64)
                        } else {
                            Interval::make_top(64)
                        };

                        // Join with existing entry (same CI-PTA conflation handling).
                        let pts = pta.points_to(dst);
                        for loc in pts {
                            sizes
                                .entry(loc)
                                .and_modify(|existing: &mut Interval| {
                                    *existing = existing.join(&size_interval);
                                })
                                .or_insert_with(|| size_interval.clone());
                        }
                    }
                }
            }
        }
    }

    sizes
}

/// Find allocation size for a pointer using PTA.
///
/// Checks the points-to set and looks up allocation sizes for each target.
/// When a LocId is not found directly (e.g., GEP-derived field location),
/// falls back to finding the base allocation by matching the same `ObjId`.
// `ptr` (pointer operand), `pta` (analysis), and `pts` (points-to set) are distinct concepts
#[allow(clippy::similar_names)]
fn find_allocation_size_with_pta(
    ptr: ValueId,
    value_sizes: &BTreeMap<ValueId, Interval>,
    loc_sizes: &BTreeMap<LocId, Interval>,
    pta: &PtaIntegration<'_>,
) -> Option<Interval> {
    // First try ValueId-based lookup (backward compatible)
    if let Some(size) = value_sizes.get(&ptr) {
        return Some(size.clone());
    }

    // Use PTA to resolve to LocIds
    let pts = pta.points_to(ptr);
    if pts.is_empty() {
        return None;
    }

    // Find sizes for all pointed-to locations
    let mut found_sizes: Vec<&Interval> = Vec::new();
    for loc in &pts {
        if let Some(size) = loc_sizes.get(loc) {
            found_sizes.push(size);
        } else {
            // Fallback: this LocId might be a GEP-derived field location.
            // Find the base allocation by matching the same ObjId across all
            // known allocation locations.
            if let Some(obj) = pta.object_of_location(*loc) {
                for (base_loc, base_size) in loc_sizes {
                    if pta.object_of_location(*base_loc) == Some(obj) {
                        found_sizes.push(base_size);
                        break;
                    }
                }
            }
        }
    }

    if found_sizes.is_empty() {
        return None;
    }

    // Join all sizes (conservative: largest possible size)
    let mut result = found_sizes[0].clone();
    for size in found_sizes.iter().skip(1) {
        result = result.join(size);
    }

    Some(result)
}

/// Fallback: if `ptr` is the result of a `Gep` instruction, resolve the
/// allocation size of the GEP's base pointer instead.
///
/// This handles the common pattern where C code decays an array to a pointer
/// via `getelementptr [N x i8], ptr %alloca, i64 0, i64 0` — the allocation
/// size is tracked for `%alloca` but not for the GEP result.
fn find_gep_base_allocation_size(
    pointer: ValueId,
    func: &AirFunction,
    value_sizes: &BTreeMap<ValueId, Interval>,
    loc_sizes: &BTreeMap<LocId, Interval>,
    pta: &PtaIntegration<'_>,
) -> Option<Interval> {
    for block in &func.blocks {
        for inst in &block.instructions {
            if matches!(&inst.op, Operation::Gep { .. }) && inst.dst == Some(pointer) {
                let base = inst.operands[0];
                return find_allocation_size_with_pta(base, value_sizes, loc_sizes, pta);
            }
        }
    }
    None
}

/// Like [`find_gep_base_allocation_size`] but returns individual per-location
/// sizes (for the TOP-size per-allocation comparison path).
fn find_gep_base_individual_sizes(
    pointer: ValueId,
    func: &AirFunction,
    value_sizes: &BTreeMap<ValueId, Interval>,
    loc_sizes: &BTreeMap<LocId, Interval>,
    pta: &PtaIntegration<'_>,
) -> Vec<Interval> {
    for block in &func.blocks {
        for inst in &block.instructions {
            if matches!(&inst.op, Operation::Gep { .. }) && inst.dst == Some(pointer) {
                let base = inst.operands[0];
                return find_individual_allocation_sizes_with_pta(
                    base,
                    value_sizes,
                    loc_sizes,
                    pta,
                );
            }
        }
    }
    Vec::new()
}

/// Find individual allocation sizes for a pointer using PTA.
///
/// Unlike [`find_allocation_size_with_pta`] which joins all sizes into one interval,
/// this returns each pointed-to allocation's size separately. This enables
/// per-allocation overflow checking, which is more precise when CI-PTA merges
/// allocations of different sizes (e.g., a 50-byte and 100-byte buffer aliased
/// by the same `char*` pointer).
// `ptr` (pointer operand), `pta` (analysis), and `pts` (points-to set) are distinct concepts
#[allow(clippy::similar_names)]
fn find_individual_allocation_sizes_with_pta(
    ptr: ValueId,
    value_sizes: &BTreeMap<ValueId, Interval>,
    loc_sizes: &BTreeMap<LocId, Interval>,
    pta: &PtaIntegration<'_>,
) -> Vec<Interval> {
    // First try ValueId-based lookup (backward compatible)
    if let Some(size) = value_sizes.get(&ptr) {
        return vec![size.clone()];
    }

    // Use PTA to resolve to LocIds
    let pts = pta.points_to(ptr);
    if pts.is_empty() {
        return Vec::new();
    }

    let mut sizes = Vec::new();
    for loc in &pts {
        if let Some(size) = loc_sizes.get(loc) {
            sizes.push(size.clone());
        } else {
            // Fallback: find base allocation by ObjId
            if let Some(obj) = pta.object_of_location(*loc) {
                for (base_loc, base_size) in loc_sizes {
                    if pta.object_of_location(*base_loc) == Some(obj) {
                        sizes.push(base_size.clone());
                        break;
                    }
                }
            }
        }
    }

    sizes
}

/// Integer overflow check using pre-computed abstract interpretation result.
fn check_integer_overflow_with_result(
    module: &AirModule,
    result: &AbstractInterpResult,
) -> Vec<NumericFinding> {
    let mut findings = Vec::new();

    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                if let Operation::BinaryOp { kind } = &inst.op {
                    if !matches!(kind, BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul) {
                        continue;
                    }
                    if inst.operands.len() < 2 {
                        continue;
                    }

                    let state = result.state_at_inst(inst.id);
                    let lhs = state
                        .and_then(|s| s.get_opt(inst.operands[0]).cloned())
                        .unwrap_or_else(|| Interval::make_top(64));
                    let rhs = state
                        .and_then(|s| s.get_opt(inst.operands[1]).cloned())
                        .unwrap_or_else(|| Interval::make_top(64));

                    if lhs.is_bottom() || rhs.is_bottom() || lhs.is_top() || rhs.is_top() {
                        continue;
                    }

                    let (result_lo, result_hi) = match kind {
                        BinaryOp::Add => lhs.add_unwrapped(&rhs),
                        BinaryOp::Sub => lhs.sub_unwrapped(&rhs),
                        BinaryOp::Mul => lhs.mul_unwrapped(&rhs),
                        _ => continue,
                    };

                    let bits = lhs.bits();
                    let target_min = signed_min(bits);
                    let target_max = signed_max(bits);

                    if result_lo >= target_min && result_hi <= target_max {
                        continue;
                    }

                    let severity = if result_lo > target_max || result_hi < target_min {
                        NumericSeverity::Error
                    } else {
                        NumericSeverity::Warning
                    };

                    let op_name = match kind {
                        BinaryOp::Add => "addition",
                        BinaryOp::Sub => "subtraction",
                        BinaryOp::Mul => "multiplication",
                        _ => "arithmetic",
                    };

                    findings.push(NumericFinding {
                        checker: NumericCheckerKind::IntegerOverflow,
                        severity,
                        cwe: 190,
                        inst_id: inst.id.to_hex(),
                        description: format!(
                            "Integer {op_name} may overflow i{bits}: [{result_lo}, {result_hi}] vs [{target_min}, {target_max}]"
                        ),
                        interval: format!("[{result_lo}, {result_hi}]"),
                        function: func.name.clone(),
                        location: (func.id, block.id),
                                affected_ptr: None,
                    });
                }
            }
        }
    }

    findings
}

/// Division by zero check using pre-computed abstract interpretation result.
fn check_division_by_zero_with_result(
    module: &AirModule,
    result: &AbstractInterpResult,
) -> Vec<NumericFinding> {
    use super::transfer::build_constant_map;

    let mut findings = Vec::new();
    let constant_map = build_constant_map(module);

    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                if let Operation::BinaryOp { kind } = &inst.op {
                    // Only check division and remainder operations
                    if !matches!(
                        kind,
                        BinaryOp::SDiv | BinaryOp::UDiv | BinaryOp::SRem | BinaryOp::URem
                    ) {
                        continue;
                    }

                    if inst.operands.len() < 2 {
                        continue;
                    }

                    // The divisor is the second operand
                    let divisor_operand = inst.operands[1];

                    // Get divisor interval: check state first, then constant_map, else top
                    let state = result.state_at_inst(inst.id);
                    let divisor = state
                        .and_then(|s| s.get_opt(divisor_operand).cloned())
                        .or_else(|| constant_map.get(&divisor_operand).cloned())
                        .unwrap_or_else(|| Interval::make_top(64));

                    // Skip unreachable code or unknown divisor
                    if divisor.is_bottom() || divisor.is_top() {
                        continue;
                    }

                    // Check if divisor interval contains zero
                    if !divisor.contains_zero() {
                        continue; // Safe: divisor cannot be zero
                    }

                    let severity = if divisor.is_singleton_zero() {
                        NumericSeverity::Error // Definitely zero
                    } else {
                        NumericSeverity::Warning // May be zero
                    };

                    let op_name = match kind {
                        BinaryOp::SDiv => "signed division",
                        BinaryOp::UDiv => "unsigned division",
                        BinaryOp::SRem => "signed remainder",
                        BinaryOp::URem => "unsigned remainder",
                        _ => "division",
                    };

                    findings.push(NumericFinding {
                        checker: NumericCheckerKind::DivisionByZero,
                        severity,
                        cwe: 369,
                        inst_id: inst.id.to_hex(),
                        description: format!("{op_name} by zero: divisor is {divisor}"),
                        interval: format!("{divisor:?}"),
                        function: func.name.clone(),
                        location: (func.id, block.id),
                        affected_ptr: None,
                    });
                }
            }
        }
    }

    findings
}

/// Shift count check using pre-computed abstract interpretation result.
fn check_shift_count_with_result(
    module: &AirModule,
    result: &AbstractInterpResult,
) -> Vec<NumericFinding> {
    use super::transfer::build_constant_map;

    let mut findings = Vec::new();
    let constant_map = build_constant_map(module);

    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                if let Operation::BinaryOp { kind } = &inst.op {
                    // Only check shift operations
                    if !matches!(kind, BinaryOp::Shl | BinaryOp::LShr | BinaryOp::AShr) {
                        continue;
                    }

                    if inst.operands.len() < 2 {
                        continue;
                    }

                    // Get the value being shifted (first operand) to determine bit-width
                    let value_operand = inst.operands[0];
                    let shift_operand = inst.operands[1];

                    let state = result.state_at_inst(inst.id);

                    // Get bit-width from the value being shifted
                    let value_interval = state
                        .and_then(|s| s.get_opt(value_operand).cloned())
                        .or_else(|| constant_map.get(&value_operand).cloned())
                        .unwrap_or_else(|| Interval::make_top(64));

                    let bits = value_interval.bits();

                    // Get shift amount interval
                    let shift_interval = state
                        .and_then(|s| s.get_opt(shift_operand).cloned())
                        .or_else(|| constant_map.get(&shift_operand).cloned())
                        .unwrap_or_else(|| Interval::make_top(64));

                    // Skip unreachable code or unknown shift amount
                    if shift_interval.is_bottom() || shift_interval.is_top() {
                        continue;
                    }

                    // Check for invalid shift amounts:
                    // 1. Negative shift count
                    // 2. Shift count >= bit-width
                    let lo = shift_interval.lo();
                    let hi = shift_interval.hi();
                    let bit_width = i128::from(bits);

                    // Check for negative shift
                    let has_negative = lo < 0;
                    // Check for shift >= bit-width
                    let has_overflow = hi >= bit_width;

                    if !has_negative && !has_overflow {
                        continue; // Shift amount is safe
                    }

                    // Determine severity based on whether the ENTIRE range is invalid
                    let entirely_negative = hi < 0;
                    let entirely_overflow = lo >= bit_width;

                    let severity = if entirely_negative || entirely_overflow {
                        NumericSeverity::Error // Definitely invalid
                    } else {
                        NumericSeverity::Warning // May be invalid
                    };

                    let op_name = match kind {
                        BinaryOp::Shl => "left shift",
                        BinaryOp::LShr => "logical right shift",
                        BinaryOp::AShr => "arithmetic right shift",
                        _ => "shift",
                    };

                    let issue = if has_negative && has_overflow {
                        format!("may be negative or >= {bits}")
                    } else if has_negative {
                        "may be negative".to_string()
                    } else {
                        format!("may be >= {bits}")
                    };

                    findings.push(NumericFinding {
                        checker: NumericCheckerKind::ShiftCount,
                        severity,
                        cwe: 682,
                        inst_id: inst.id.to_hex(),
                        description: format!(
                            "invalid {op_name} count: shift amount {shift_interval} {issue}"
                        ),
                        interval: format!("{shift_interval:?}"),
                        function: func.name.clone(),
                        location: (func.id, block.id),
                        affected_ptr: None,
                    });
                }
            }
        }
    }

    findings
}

/// Check for memcpy/memmove/memset buffer overflows (CWE-120, CWE-122).
///
/// At each memcpy/memmove/memset intrinsic, checks whether the size operand
/// exceeds the known allocation size of the destination buffer.
///
/// This complements `check_buffer_overflow` which detects GEP-based overflows.
/// Memcpy-style overflows occur when:
/// - `memcpy(dest, src, size)` with `size > allocation_size(dest)`
/// - `memset(dest, val, size)` with `size > allocation_size(dest)`
#[must_use]
pub fn check_memcpy_overflow(
    module: &AirModule,
    config: &AbstractInterpConfig,
) -> NumericCheckResult {
    let result = solve_abstract_interp(module, config);
    let findings = check_memcpy_overflow_impl(module, &result, None);
    NumericCheckResult {
        findings,
        absint_result: Some(result),
    }
}

/// Check for memcpy-style buffer overflows using function specifications.
///
/// This extends `check_memcpy_overflow` to also detect overflows in:
/// - `CallDirect` calls to functions with `semantic: byte_count` parameters
/// - `strcpy`, `strncpy`, `sprintf`, etc. when specs define their size parameters
#[must_use]
pub fn check_memcpy_overflow_with_specs(
    module: &AirModule,
    config: &AbstractInterpConfig,
    specs: &SpecRegistry,
) -> NumericCheckResult {
    let result = solve_abstract_interp(module, config);
    let findings = check_memcpy_overflow_impl(module, &result, Some(specs));
    NumericCheckResult {
        findings,
        absint_result: Some(result),
    }
}

/// Check for memcpy-style buffer overflows with PTA and function specifications.
///
/// Uses PTA to resolve pointers to allocation sites instead of intraprocedural
/// store→load chain heuristics. This is more precise at `-O0` where LLVM
/// generates store/load pairs through alloca'd addresses.
#[must_use]
pub fn check_memcpy_overflow_with_pta_and_specs(
    module: &AirModule,
    config: &AbstractInterpConfig,
    pta: &PtaIntegration<'_>,
    specs: &SpecRegistry,
) -> NumericCheckResult {
    let result = solve_abstract_interp(module, config);
    let findings = check_memcpy_overflow_with_pta_impl(module, &result, pta, specs);
    NumericCheckResult {
        findings,
        absint_result: Some(result),
    }
}

/// Check for memcpy-style buffer overflows using a pre-computed `AbstractInterpResult`.
///
/// This avoids re-running the abstract interpretation solver when the caller
/// already has a result (e.g., from a prior `check_buffer_overflow` call).
/// Accepts optional `SpecRegistry` for spec-defined copy functions.
#[must_use]
pub fn check_memcpy_overflow_with_result(
    module: &AirModule,
    result: &AbstractInterpResult,
    specs: Option<&SpecRegistry>,
) -> Vec<NumericFinding> {
    check_memcpy_overflow_impl(module, result, specs)
}

/// Memcpy overflow check implementation.
///
/// When `specs` is provided, also checks `CallDirect` calls to functions
/// with `semantic: byte_count` or `semantic: max_length` parameters.
// NOTE: This function checks both intrinsic memcpy/memset operations and
// spec-defined copy/string functions. Splitting would separate related size
// comparison logic across multiple functions with no readability benefit.
#[allow(clippy::too_many_lines)]
fn check_memcpy_overflow_impl(
    module: &AirModule,
    result: &AbstractInterpResult,
    specs: Option<&SpecRegistry>,
) -> Vec<NumericFinding> {
    use super::transfer::build_constant_map;

    let mut findings = Vec::new();
    let constant_map = build_constant_map(module);

    // Build a map from ValueId → allocation size for all allocations
    // (HeapAlloc, Alloca, and fixed-size array allocations from types)
    let alloca_size_map = build_alloca_size_map(module);
    let alloc_sizes = extract_allocation_sizes(module, result, &alloca_size_map);

    // Build a map from pointer ValueId → base allocation ValueId
    // by following Load/Copy/GEP chains
    let pointer_origins = build_pointer_origins(module);

    // Build function name map for CallDirect lookups
    let func_names: BTreeMap<FunctionId, &str> = module
        .functions
        .iter()
        .map(|f| (f.id, f.name.as_str()))
        .collect();

    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                // Check memcpy/memset intrinsics
                let is_memcpy = matches!(&inst.op, Operation::Memcpy);
                let is_memset = matches!(&inst.op, Operation::Memset);

                // Check CallDirect to spec-defined functions with byte_count semantic.
                // Also handle wmemcpy/wmemset/wmemmove by name (not in specs).
                let spec_info = if let Operation::CallDirect { callee } = &inst.op {
                    if let Some(callee_name) = func_names.get(callee) {
                        let from_specs = specs
                            .and_then(|s| get_spec_copy_info(callee_name, s, inst.operands.len()));
                        from_specs
                            .or_else(|| get_builtin_copy_info(callee_name, inst.operands.len()))
                    } else {
                        None
                    }
                } else {
                    None
                };

                // Check for string copy functions (strcpy, strcat) with copies_string semantic.
                // Note: this compares ALLOCATION sizes, not content sizes. This can cause
                // false positives when source buffers are larger but only partially filled.
                let string_copy_info = if spec_info.is_none() && !is_memcpy && !is_memset {
                    if let Operation::CallDirect { callee } = &inst.op {
                        if let Some(specs) = specs {
                            if let Some(callee_name) = func_names.get(callee) {
                                get_string_copy_info(callee_name, specs, inst.operands.len())
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };

                // Handle string copy separately (src-vs-dst allocation size comparison)
                if let Some((dst_idx, src_idx, ref callee_name)) = string_copy_info {
                    if inst.operands.len() > dst_idx && inst.operands.len() > src_idx {
                        let dst_op = inst.operands[dst_idx];
                        let src_op = inst.operands[src_idx];

                        let dst_size = find_dest_allocation_size(
                            dst_op,
                            &pointer_origins,
                            &alloc_sizes,
                            result,
                            &constant_map,
                        );
                        let src_size = find_dest_allocation_size(
                            src_op,
                            &pointer_origins,
                            &alloc_sizes,
                            result,
                            &constant_map,
                        );

                        if let (Some(dst_alloc), Some(src_alloc)) = (dst_size, src_size) {
                            if !dst_alloc.is_top()
                                && !dst_alloc.is_bottom()
                                && !src_alloc.is_top()
                                && !src_alloc.is_bottom()
                            {
                                let dst_hi = dst_alloc.hi();
                                let src_lo = src_alloc.lo();
                                if dst_hi > 0 && src_lo > dst_hi {
                                    findings.push(NumericFinding {
                                        checker: NumericCheckerKind::BufferOverflow,
                                        severity: NumericSeverity::Warning,
                                        cwe: 120,
                                        inst_id: inst.id.to_hex(),
                                        description: format!(
                                            "{callee_name} may overflow: source allocation size {src_alloc} exceeds destination size {dst_alloc}"
                                        ),
                                        interval: format!("{src_alloc:?}"),
                                        function: func.name.clone(),
                                        location: (func.id, block.id),
                                        affected_ptr: Some(dst_op),
                                    });
                                }
                            }
                        }
                    }
                    continue;
                }

                // Skip if not a memory operation
                if !is_memcpy && !is_memset && spec_info.is_none() {
                    continue;
                }

                // Determine dest and size operand indices
                let (dest_idx, size_idx, op_name) = if is_memcpy || is_memset {
                    // Intrinsics: operands are [dest, src/val, size, is_volatile]
                    if inst.operands.len() < 3 {
                        continue;
                    }
                    let name = if is_memcpy {
                        "memcpy/memmove"
                    } else {
                        "memset"
                    };
                    (0usize, 2usize, name)
                } else if let Some((dest, size, name)) = &spec_info {
                    (*dest, *size, name.as_str())
                } else {
                    continue;
                };

                if inst.operands.len() <= dest_idx || inst.operands.len() <= size_idx {
                    continue;
                }

                let dest_operand = inst.operands[dest_idx];
                let size_operand = inst.operands[size_idx];

                // Get the copy size interval
                let state = result.state_at_inst(inst.id);
                let size_interval = state
                    .and_then(|s| s.get_opt(size_operand).cloned())
                    .or_else(|| constant_map.get(&size_operand).cloned())
                    .unwrap_or_else(|| Interval::make_top(64));

                // Skip if size is unknown or unreachable
                if size_interval.is_bottom() || size_interval.is_top() {
                    continue;
                }

                // Try to find the allocation size for the destination pointer
                // First, follow pointer chains to find the origin allocation
                let dest_alloc_size = find_dest_allocation_size(
                    dest_operand,
                    &pointer_origins,
                    &alloc_sizes,
                    result,
                    &constant_map,
                );

                if let Some(alloc_size) = dest_alloc_size {
                    if !alloc_size.is_top() && !alloc_size.is_bottom() {
                        // Compare: if copy size may exceed allocation size, report
                        let alloc_hi = alloc_size.hi();
                        let size_lo = size_interval.lo();
                        let size_hi = size_interval.hi();

                        if alloc_hi > 0 && size_hi > alloc_hi {
                            let severity = if size_lo > alloc_hi {
                                NumericSeverity::Error // Copy always overflows
                            } else {
                                NumericSeverity::Warning // Copy may overflow
                            };

                            findings.push(NumericFinding {
                                checker: NumericCheckerKind::BufferOverflow,
                                severity,
                                cwe: 120,
                                inst_id: inst.id.to_hex(),
                                description: format!(
                                    "{op_name} may overflow: copy size {size_interval} exceeds buffer size {alloc_size}"
                                ),
                                interval: format!("{size_interval:?}"),
                                function: func.name.clone(),
                                location: (func.id, block.id),
                                affected_ptr: Some(dest_operand),
                            });
                        }
                    }
                }
            }
        }
    }

    findings
}

/// Memcpy overflow check with PTA integration.
///
/// Uses PTA to resolve pointers to allocation sites instead of intraprocedural
/// store→load chain heuristics (`build_pointer_origins`). At `-O0`, LLVM
/// generates store/load pairs through alloca'd addresses that break the
/// heuristic approach, but PTA handles these patterns correctly.
// NOTE: This function mirrors `check_memcpy_overflow_impl` but uses PTA for
// pointer resolution. Splitting further would obscure the parallel structure.
#[allow(clippy::too_many_lines, clippy::similar_names)]
fn check_memcpy_overflow_with_pta_impl(
    module: &AirModule,
    result: &AbstractInterpResult,
    pta: &PtaIntegration<'_>,
    specs: &SpecRegistry,
) -> Vec<NumericFinding> {
    use super::transfer::{build_constant_map, build_obj_type_map};

    let mut findings = Vec::new();
    let constant_map = build_constant_map(module);

    // Build allocation size maps using PTA for pointer resolution
    let alloca_size_map = build_alloca_size_map(module);
    let alloc_sizes = extract_allocation_sizes(module, result, &alloca_size_map);
    let loc_alloc_sizes = build_loc_allocation_sizes(module, result, pta);

    // Build field-level allocation sizes for struct-aware overflow detection
    let obj_type_map = build_obj_type_map(module);
    let field_alloc_sizes = build_field_allocation_sizes(module, pta, &obj_type_map);

    // Build function name map for CallDirect lookups
    let func_names: BTreeMap<FunctionId, &str> = module
        .functions
        .iter()
        .map(|f| (f.id, f.name.as_str()))
        .collect();

    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                // Check memcpy/memset intrinsics
                let is_memcpy = matches!(&inst.op, Operation::Memcpy);
                let is_memset = matches!(&inst.op, Operation::Memset);

                // Check CallDirect to spec-defined functions with byte_count/max_length/buffer_size
                // Also handle wmemcpy/wmemset/wmemmove directly by name (not LLVM intrinsics,
                // may not be in specs)
                let spec_info = if let Operation::CallDirect { callee } = &inst.op {
                    if let Some(callee_name) = func_names.get(callee) {
                        get_spec_copy_info(callee_name, specs, inst.operands.len())
                            .or_else(|| get_builtin_copy_info(callee_name, inst.operands.len()))
                    } else {
                        None
                    }
                } else {
                    None
                };

                // Detect strcat/strncat append overflow.
                //
                // `strcat(dest, src)` appends `strlen(src)` bytes to the existing
                // content of `dest`.  If the combined length exceeds the destination
                // allocation, this is a buffer overflow.
                //
                // Strategy:
                // 1. If src is a global string literal, use the literal length.
                // 2. Scan backwards in the same block for a preceding `strcpy` or
                //    `strcat` to the same dest to estimate initial content length.
                // 3. Fall back to comparing src allocation size vs dest allocation
                //    size (strcat always appends, so src_alloc >= dest_alloc is
                //    suspicious).
                if spec_info.is_none() && !is_memcpy && !is_memset {
                    if let Operation::CallDirect { callee } = &inst.op {
                        if let Some(callee_name) = func_names.get(callee) {
                            if *callee_name == "strcat" || *callee_name == "strncat" {
                                check_strcat_overflow(
                                    inst,
                                    callee_name,
                                    block,
                                    func,
                                    module,
                                    &alloc_sizes,
                                    &loc_alloc_sizes,
                                    pta,
                                    &mut findings,
                                );
                                continue;
                            }
                        }
                    }
                }

                // Skip if not a memory operation
                if !is_memcpy && !is_memset && spec_info.is_none() {
                    continue;
                }

                // Determine dest and size operand indices
                let (dest_idx, size_idx, op_name, is_wide_char) = if is_memcpy || is_memset {
                    if inst.operands.len() < 3 {
                        continue;
                    }
                    let name = if is_memcpy {
                        "memcpy/memmove"
                    } else {
                        "memset"
                    };
                    (0usize, 2usize, name, false)
                } else if let Some((dest, size, ref name)) = spec_info {
                    // Detect wide-char functions (swprintf etc.) for size multiplier
                    let wide = name.starts_with("sw") || name.contains("wchar");
                    (dest, size, name.as_str(), wide)
                } else {
                    continue;
                };

                if inst.operands.len() <= dest_idx || inst.operands.len() <= size_idx {
                    continue;
                }

                let dest_operand = inst.operands[dest_idx];
                let size_operand = inst.operands[size_idx];

                // Get the copy size interval
                let state = result.state_at_inst(inst.id);
                let size_interval = state
                    .and_then(|s| s.get_opt(size_operand).cloned())
                    .or_else(|| constant_map.get(&size_operand).cloned())
                    .unwrap_or_else(|| Interval::make_top(64));

                // Use PTA to resolve dest allocation size (with GEP-base fallback)
                let mut dest_alloc_size = find_allocation_size_with_pta(
                    dest_operand,
                    &alloc_sizes,
                    &loc_alloc_sizes,
                    pta,
                )
                .or_else(|| {
                    find_gep_base_allocation_size(
                        dest_operand,
                        func,
                        &alloc_sizes,
                        &loc_alloc_sizes,
                        pta,
                    )
                });

                // Field-aware refinement: if the dest pointer has field GEP targets
                // in the abstract state, use the field-level remaining size instead
                // of the total allocation size.
                if let Some(st) = state {
                    if let Some(field_targets) = st.resolve_field_gep(dest_operand) {
                        for &(loc, offset) in field_targets {
                            if let Some(field_size) = field_alloc_sizes.get(&(loc, offset)) {
                                // Use the more precise field-level size
                                dest_alloc_size = Some(field_size.clone());
                                break;
                            }
                        }
                    }
                }

                // Phase F: When size is TOP (e.g., from strlen), compare allocations directly
                if size_interval.is_top() && !is_memset {
                    // For memcpy/memmove with unknown size, check src vs dst allocation.
                    // Use per-allocation comparison: if ANY individual destination
                    // allocation is smaller than the source, report may-overflow.
                    // This handles CI-PTA imprecision where join([200],[400])=[200,400]
                    // would mask the overflow for the 200-byte allocation.
                    let src_idx_for_top = if is_memcpy { Some(1usize) } else { None };
                    if let Some(si) = src_idx_for_top {
                        if inst.operands.len() > si {
                            let src_operand = inst.operands[si];
                            // For destination: prefer function-local allocation size
                            // over PTA-resolved sizes. This avoids CI-PTA false positives
                            // where dest in goodG2B() (malloc(40)) aliases dest in _bad()
                            // (malloc(10)), causing min_dst=10 and a spurious finding.
                            let dst_sizes = if let Some(local) = alloc_sizes.get(&dest_operand) {
                                vec![local.clone()]
                            } else {
                                let mut sizes = find_individual_allocation_sizes_with_pta(
                                    dest_operand,
                                    &alloc_sizes,
                                    &loc_alloc_sizes,
                                    pta,
                                );
                                if sizes.is_empty() {
                                    sizes = find_gep_base_individual_sizes(
                                        dest_operand,
                                        func,
                                        &alloc_sizes,
                                        &loc_alloc_sizes,
                                        pta,
                                    );
                                }
                                sizes
                            };
                            let src_sizes = {
                                let mut sizes = find_individual_allocation_sizes_with_pta(
                                    src_operand,
                                    &alloc_sizes,
                                    &loc_alloc_sizes,
                                    pta,
                                );
                                if sizes.is_empty() {
                                    sizes = find_gep_base_individual_sizes(
                                        src_operand,
                                        func,
                                        &alloc_sizes,
                                        &loc_alloc_sizes,
                                        pta,
                                    );
                                }
                                sizes
                            };

                            // Check if any source allocation exceeds any destination allocation
                            let max_src = src_sizes
                                .iter()
                                .filter(|s| !s.is_top() && !s.is_bottom())
                                .map(super::interval::Interval::hi)
                                .max();
                            let min_dst = dst_sizes
                                .iter()
                                .filter(|s| !s.is_top() && !s.is_bottom() && s.hi() > 0)
                                .map(super::interval::Interval::hi)
                                .min();

                            if let (Some(max_s), Some(min_d)) = (max_src, min_dst) {
                                if max_s > min_d {
                                    findings.push(NumericFinding {
                                        checker: NumericCheckerKind::BufferOverflow,
                                        severity: NumericSeverity::Warning,
                                        cwe: 120,
                                        inst_id: inst.id.to_hex(),
                                        description: format!(
                                            "{op_name} may overflow: source allocation [{max_s}] exceeds destination [{min_d}] (copy size unknown)"
                                        ),
                                        interval: "TOP".to_string(),
                                        function: func.name.clone(),
                                        location: (func.id, block.id),
                                        affected_ptr: Some(dest_operand),
                                    });
                                }
                            }
                        }
                    }
                    continue;
                }

                // Skip if size is unreachable
                if size_interval.is_bottom() {
                    continue;
                }

                // For wide-char buffer_size, multiply by sizeof(wchar_t) = 4
                let effective_size = if is_wide_char && !size_interval.is_top() {
                    let wchar_size = Interval::singleton(4, size_interval.bits());
                    size_interval.mul(&wchar_size)
                } else {
                    size_interval.clone()
                };

                if let Some(alloc_size) = &dest_alloc_size {
                    if !alloc_size.is_top() && !alloc_size.is_bottom() {
                        let alloc_hi = alloc_size.hi();
                        let size_lo = effective_size.lo();
                        let size_hi = effective_size.hi();

                        if alloc_hi > 0 && size_hi > alloc_hi {
                            let severity = if size_lo > alloc_hi {
                                NumericSeverity::Error
                            } else {
                                NumericSeverity::Warning
                            };

                            findings.push(NumericFinding {
                                checker: NumericCheckerKind::BufferOverflow,
                                severity,
                                cwe: 120,
                                inst_id: inst.id.to_hex(),
                                description: format!(
                                    "{op_name} may overflow: copy size {effective_size} exceeds buffer size {alloc_size}"
                                ),
                                interval: format!("{effective_size:?}"),
                                function: func.name.clone(),
                                location: (func.id, block.id),
                                affected_ptr: Some(dest_operand),
                            });
                        }
                    }
                }

                // Phase E: Source overread detection for memcpy/memmove
                if is_memcpy && inst.operands.len() > 1 && !effective_size.is_top() {
                    let src_operand = inst.operands[1];
                    let src_alloc_size = find_allocation_size_with_pta(
                        src_operand,
                        &alloc_sizes,
                        &loc_alloc_sizes,
                        pta,
                    )
                    .or_else(|| {
                        find_gep_base_allocation_size(
                            src_operand,
                            func,
                            &alloc_sizes,
                            &loc_alloc_sizes,
                            pta,
                        )
                    });
                    if let Some(src_alloc) = src_alloc_size {
                        if !src_alloc.is_top()
                            && !src_alloc.is_bottom()
                            && src_alloc.hi() > 0
                            && effective_size.lo() > src_alloc.hi()
                        {
                            findings.push(NumericFinding {
                                checker: NumericCheckerKind::BufferOverflow,
                                severity: NumericSeverity::Warning,
                                cwe: 126,
                                inst_id: inst.id.to_hex(),
                                description: format!(
                                    "{op_name} source overread: copy size {effective_size} exceeds source buffer size {src_alloc}"
                                ),
                                interval: format!("{effective_size:?}"),
                                function: func.name.clone(),
                                location: (func.id, block.id),
                                affected_ptr: Some(src_operand),
                            });
                        }
                    }
                }
            }
        }
    }

    findings
}

/// Get the byte length of a global string constant (including null terminator).
///
/// Looks up a `ValueId` in the module's globals to find a string constant
/// initializer (e.g., `@.str = constant [11 x i8] c"worldworld\00"`).
/// Returns the byte length including the null terminator, which is
/// `string_content.len() + 1` since the LLVM frontend strips the trailing
/// null when converting to `Constant::String`.
fn get_global_string_length(operand: ValueId, module: &AirModule) -> Option<usize> {
    for global in &module.globals {
        if global.id == operand {
            if let Some(Constant::String { value }) = &global.init {
                // The frontend strips the null terminator, so add 1 for the
                // actual C string length including '\0'.
                return Some(value.len() + 1);
            }
            // Also handle Aggregate of byte constants (e.g., c"Hi\00\00...")
            if let Some(Constant::Aggregate { elements }) = &global.init {
                // Count elements up to (and including) the first null byte
                // to get strlen + 1. Aggregate elements are individual bytes
                // for char arrays.
                let mut content_len = 0usize;
                for elem in elements {
                    if let Constant::Int { value, .. } = elem {
                        if *value == 0 {
                            return Some(content_len + 1);
                        }
                        content_len += 1;
                    } else {
                        // Non-integer element — not a byte array
                        return None;
                    }
                }
                // No null terminator found — return full aggregate length
                return Some(elements.len());
            }
        }
    }
    None
}

/// Check for `strcat`/`strncat` append overflow (CWE-120).
///
/// Detects overflow when `strcat(dest, src)` appends to a buffer that already
/// contains data from a preceding `strcpy` or `strcat`. Also detects cases
/// where the source allocation is larger than the destination, which implies
/// overflow since `strcat` appends to existing content.
// NOTE: This function implements strcat overflow detection with backward scan
// for preceding string operations, src literal lookup, and PTA-based allocation
// comparison. The sequential checks are easier to follow as one function.
#[allow(clippy::too_many_arguments)]
fn check_strcat_overflow(
    inst: &Instruction,
    callee_name: &str,
    block: &AirBlock,
    func: &AirFunction,
    module: &AirModule,
    alloc_sizes: &BTreeMap<ValueId, Interval>,
    loc_alloc_sizes: &BTreeMap<LocId, Interval>,
    pta: &PtaIntegration<'_>,
    findings: &mut Vec<NumericFinding>,
) {
    if inst.operands.len() < 2 {
        return;
    }

    let dest_operand = inst.operands[0];
    let src_operand = inst.operands[1];

    // Get dest allocation size via PTA (with GEP-base fallback)
    let dest_alloc = find_allocation_size_with_pta(dest_operand, alloc_sizes, loc_alloc_sizes, pta)
        .or_else(|| {
            find_gep_base_allocation_size(dest_operand, func, alloc_sizes, loc_alloc_sizes, pta)
        });
    let Some(dest_alloc) = dest_alloc else {
        return;
    };
    if dest_alloc.is_top() || dest_alloc.is_bottom() || dest_alloc.hi() <= 0 {
        return;
    }
    let dest_size = dest_alloc.hi();

    // Try to get the source string literal length from globals
    let src_literal_len = get_global_string_length(src_operand, module);

    // Look backwards in the block for preceding strcpy/strcat calls to the
    // same dest pointer to estimate accumulated content length.
    let initial_content = estimate_initial_content(inst, block, dest_operand, module, pta);

    // Case A: Source is a known string literal — compute exact overflow
    if let Some(append_len) = src_literal_len {
        // strlen(src) = append_len - 1 (excluding null)
        // After strcat: initial_content + strlen(src) + 1 (for null) = total bytes
        let total_bytes = initial_content + append_len - 1 + 1;
        // total_bytes = initial_content + append_len
        // (the -1 for shared null cancels with +1 for the final null)
        let total_bytes_val = i128::try_from(total_bytes).unwrap_or(i128::MAX);
        if total_bytes_val > dest_size {
            let severity = NumericSeverity::Error;
            findings.push(NumericFinding {
                checker: NumericCheckerKind::BufferOverflow,
                severity,
                cwe: 120,
                inst_id: inst.id.to_hex(),
                description: format!(
                    "{callee_name} overflow: appending {append_len}-byte string to buffer \
                     with {initial_content} bytes of existing content requires {total_bytes} \
                     bytes, exceeds allocation of {dest_size}"
                ),
                interval: format!("[{total_bytes_val}]"),
                function: func.name.clone(),
                location: (func.id, block.id),
                affected_ptr: Some(dest_operand),
            });
            return;
        }
    }

    // Case B: Source is not a literal — compare allocation sizes.
    // Since strcat appends (dest already has content), if src_alloc >= dest_alloc
    // then overflow is likely. Even src_alloc > dest_alloc/2 is suspicious, but
    // we conservatively flag when src_alloc >= dest_alloc.
    let src_alloc = find_allocation_size_with_pta(src_operand, alloc_sizes, loc_alloc_sizes, pta)
        .or_else(|| {
            find_gep_base_allocation_size(src_operand, func, alloc_sizes, loc_alloc_sizes, pta)
        });
    if let Some(src_alloc) = src_alloc {
        if !src_alloc.is_top() && !src_alloc.is_bottom() && src_alloc.hi() > 0 {
            let src_size = src_alloc.hi();
            // strcat appends to existing content, so if the source allocation
            // is >= dest allocation, overflow is almost certain (dest already
            // has at least 1 byte of content if strcat is called after strcpy
            // or initialization).
            if src_size >= dest_size {
                findings.push(NumericFinding {
                    checker: NumericCheckerKind::BufferOverflow,
                    severity: NumericSeverity::Warning,
                    cwe: 120,
                    inst_id: inst.id.to_hex(),
                    description: format!(
                        "{callee_name} may overflow: source allocation [{src_size}] >= \
                         destination allocation [{dest_size}] (strcat appends to existing content)"
                    ),
                    interval: format!("[{src_size}]"),
                    function: func.name.clone(),
                    location: (func.id, block.id),
                    affected_ptr: Some(dest_operand),
                });
            }
        }
    }
}

/// Estimate the byte count of existing content in a buffer before a `strcat` call.
///
/// Scans backwards through the block's instructions looking for preceding `strcpy`
/// or `strcat` calls to the same destination pointer. Accumulates the known string
/// literal lengths to estimate how many bytes of content are already in the buffer.
///
/// Returns the estimated strlen (content bytes, NOT including the null terminator).
fn estimate_initial_content(
    strcat_inst: &Instruction,
    block: &AirBlock,
    dest_operand: ValueId,
    module: &AirModule,
    pta: &PtaIntegration<'_>,
) -> usize {
    let mut accumulated: usize = 0;

    // Find the position of the current strcat instruction in the block
    let Some(strcat_pos) = block
        .instructions
        .iter()
        .position(|i| i.id == strcat_inst.id)
    else {
        return 0;
    };

    // Scan backwards through preceding instructions
    for prev_inst in block.instructions[..strcat_pos].iter().rev() {
        // Handle Memcpy intrinsics (buffer initialization via llvm.memcpy from
        // a global constant, e.g., `char buffer[15] = "Start"`).
        if matches!(&prev_inst.op, Operation::Memcpy) && prev_inst.operands.len() >= 2 {
            let prev_dest = prev_inst.operands[0];
            let prev_src = prev_inst.operands[1];

            let same_dest = prev_dest == dest_operand || {
                let pts_a = pta.points_to(prev_dest);
                let pts_b = pta.points_to(dest_operand);
                !pts_a.is_empty() && pts_a.iter().any(|loc| pts_b.contains(loc))
            };

            if same_dest {
                if let Some(src_len) = get_global_string_length(prev_src, module) {
                    // memcpy from a global string initializer sets the base
                    // content.  Any strcat contributions already accumulated
                    // (from instructions chronologically AFTER this one) are
                    // added on top of the base.
                    accumulated += src_len - 1;
                    break; // Base content established — stop scanning
                }
            }
            continue;
        }

        let Operation::CallDirect { callee } = &prev_inst.op else {
            continue;
        };

        // Find the called function name
        let prev_name = module
            .functions
            .iter()
            .find(|f| f.id == *callee)
            .map(|f| f.name.as_str());
        let Some(prev_name) = prev_name else {
            continue;
        };

        if prev_name != "strcpy" && prev_name != "strcat" {
            continue;
        }
        if prev_inst.operands.len() < 2 {
            continue;
        }

        let prev_dest = prev_inst.operands[0];
        let prev_src = prev_inst.operands[1];

        // Check if the previous call targets the same dest pointer.
        // Use PTA for alias comparison since GEP produces different SSA values
        // for the same base pointer.
        let same_dest = prev_dest == dest_operand || {
            let pts_a = pta.points_to(prev_dest);
            let pts_b = pta.points_to(dest_operand);
            !pts_a.is_empty() && pts_a.iter().any(|loc| pts_b.contains(loc))
        };

        if !same_dest {
            continue;
        }

        // Get the source string literal length
        if let Some(src_len) = get_global_string_length(prev_src, module) {
            let content_len = src_len - 1; // strlen (without null)
            if prev_name == "strcpy" {
                // strcpy sets the base content.  Strcat contributions already
                // accumulated (chronologically later) are added on top.
                accumulated += content_len;
                break; // Base content established — stop scanning
            }
            // strcat appends
            accumulated += content_len;
        }
    }

    accumulated
}

/// Get copy operation info from function spec.
///
/// Returns `(dest_param_index, size_param_index, function_name)` if the function
/// has a parameter with `semantic: byte_count`, `semantic: max_length`, or
/// `semantic: buffer_size`.
fn get_spec_copy_info(
    callee_name: &str,
    specs: &SpecRegistry,
    operand_count: usize,
) -> Option<(usize, usize, String)> {
    let spec = specs.lookup(callee_name)?;

    // Find the size parameter (byte_count, max_length, or buffer_size semantic)
    let size_param = spec.params.iter().find(|p| {
        p.semantic.as_deref() == Some("byte_count")
            || p.semantic.as_deref() == Some("max_length")
            || p.semantic.as_deref() == Some("buffer_size")
    })?;

    let size_idx = size_param.index as usize;

    // Find the destination parameter (modifies: true)
    // Default to param 0 if not specified
    let dest_idx = spec
        .params
        .iter()
        .find(|p| p.modifies == Some(true))
        .map_or(0, |p| p.index as usize);

    // Validate indices are in range
    if size_idx >= operand_count || dest_idx >= operand_count {
        return None;
    }

    Some((dest_idx, size_idx, callee_name.to_string()))
}

/// Get string copy operation info from function spec.
///
/// Returns `(dest_param_index, src_param_index, function_name)` if the function
/// has a parameter with `semantic: copies_string`.
fn get_string_copy_info(
    callee_name: &str,
    specs: &SpecRegistry,
    operand_count: usize,
) -> Option<(usize, usize, String)> {
    let spec = specs.lookup(callee_name)?;

    // Find the source parameter (copies_string semantic)
    let src_param = spec
        .params
        .iter()
        .find(|p| p.semantic.as_deref() == Some("copies_string"))?;

    let src_idx = src_param.index as usize;

    // Find the destination parameter (modifies: true)
    // Default to param 0 if not specified
    let dest_idx = spec
        .params
        .iter()
        .find(|p| p.modifies == Some(true))
        .map_or(0, |p| p.index as usize);

    // Validate indices are in range
    if src_idx >= operand_count || dest_idx >= operand_count {
        return None;
    }

    Some((dest_idx, src_idx, callee_name.to_string()))
}

/// Get copy info for built-in memory functions that may not be in specs.
///
/// Handles wide-character memory functions (`wmemcpy`, `wmemset`, `wmemmove`)
/// that are C library functions (not LLVM intrinsics) and may not have spec YAML.
/// Returns `(dest_param_index, size_param_index, function_name)`.
fn get_builtin_copy_info(
    callee_name: &str,
    operand_count: usize,
) -> Option<(usize, usize, String)> {
    // wmemcpy(dst, src, count) — count is in wchar_t units, but the LLVM call
    // passes the byte count after implicit sizeof(wchar_t) multiplication
    match callee_name {
        "wmemcpy" | "wmemmove" | "wmemset" if operand_count >= 3 => {
            Some((0, 2, callee_name.to_string()))
        }
        _ => None,
    }
}

/// Build a map from pointer ValueId to its origin (where the pointer comes from).
///
/// Follows Load, Copy, GEP, and Store→Load chains to find the base allocation.
/// This is a simplified intraprocedural pointer tracking.
fn build_pointer_origins(module: &AirModule) -> BTreeMap<ValueId, ValueId> {
    let mut origins = BTreeMap::new();
    // Track what was last stored to each address: address → stored value
    let mut stored_values: BTreeMap<ValueId, ValueId> = BTreeMap::new();

    // First pass: collect all stores and basic origins
    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                // Track stores: store value, ptr → stored_values[ptr] = value
                if let Operation::Store = &inst.op {
                    if inst.operands.len() >= 2 {
                        let value = inst.operands[0];
                        let ptr = inst.operands[1];
                        stored_values.insert(ptr, value);
                    }
                }

                let Some(dst) = inst.dst else { continue };

                match &inst.op {
                    // Copy/GEP: result is derived from first operand
                    Operation::Copy | Operation::Gep { .. } => {
                        if let Some(&src) = inst.operands.first() {
                            origins.insert(dst, src);
                        }
                    }
                    // HeapAlloc/Alloca: these are origins themselves
                    Operation::HeapAlloc { .. } | Operation::Alloca { .. } => {
                        origins.insert(dst, dst);
                    }
                    _ => {}
                }
            }
        }
    }

    // Second pass: resolve Load instructions using stored_values
    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                if let Operation::Load = &inst.op {
                    if let Some(dst) = inst.dst {
                        if let Some(&ptr) = inst.operands.first() {
                            // If we know what was stored at ptr, the load result
                            // points to that stored value
                            if let Some(&stored_val) = stored_values.get(&ptr) {
                                origins.insert(dst, stored_val);
                            } else {
                                // Fallback: record the pointer being loaded from
                                origins.insert(dst, ptr);
                            }
                        }
                    }
                }
            }
        }
    }

    origins
}

/// Find the allocation size for a destination pointer by following pointer origins.
fn find_dest_allocation_size(
    dest: ValueId,
    origins: &BTreeMap<ValueId, ValueId>,
    alloc_sizes: &BTreeMap<ValueId, Interval>,
    _result: &AbstractInterpResult,
    _constant_map: &BTreeMap<ValueId, Interval>,
) -> Option<Interval> {
    // Walk the origin chain to find the base allocation
    let mut current = dest;
    let mut visited = std::collections::BTreeSet::new();

    while let Some(&origin) = origins.get(&current) {
        if visited.contains(&origin) {
            break; // Cycle detected
        }
        visited.insert(current);

        // Check if this origin has a known allocation size
        if let Some(size) = alloc_sizes.get(&origin) {
            return Some(size.clone());
        }

        // If origin points to itself, it's a base allocation without tracked size
        if origin == current {
            break;
        }

        current = origin;
    }

    // Check final current value
    alloc_sizes.get(&current).cloned()
}

/// Build a map of alloca `ValueId`s to their known sizes from `size_bytes`.
///
/// This pre-computes sizes before abstract interpretation so that even
/// allocas promoted by mem2reg retain their original size information.
fn build_alloca_size_map(module: &AirModule) -> BTreeMap<ValueId, Interval> {
    let mut map = BTreeMap::new();
    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                if let Operation::Alloca {
                    size_bytes: Some(size),
                } = &inst.op
                {
                    if let Some(dst) = inst.dst {
                        map.insert(dst, Interval::singleton(i128::from(*size), 64));
                    }
                }
            }
        }
    }
    map
}

/// Extract allocation sizes from `HeapAlloc` and `Alloca` instructions.
///
/// Maps the result `ValueId` of each allocation to its size interval.
/// Uses both abstract interpretation state, constant map, and the
/// pre-computed `alloca_size_map` to resolve sizes.
fn extract_allocation_sizes(
    module: &AirModule,
    result: &AbstractInterpResult,
    alloca_size_map: &BTreeMap<ValueId, Interval>,
) -> BTreeMap<ValueId, Interval> {
    use super::transfer::build_constant_map;

    let mut sizes = BTreeMap::new();
    let constant_map = build_constant_map(module);

    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                if let Operation::HeapAlloc { .. } = &inst.op {
                    if let Some(dst) = inst.dst {
                        // The first operand is typically the size
                        if let Some(&size_operand) = inst.operands.first() {
                            let state = result.state_at_inst(inst.id);
                            // Try state first, then constant map, then fallback to top
                            let size_interval = state
                                .and_then(|s| s.get_opt(size_operand).cloned())
                                .or_else(|| constant_map.get(&size_operand).cloned())
                                .unwrap_or_else(|| Interval::make_top(64));

                            sizes.insert(dst, size_interval);
                        }
                    }
                }

                // Also track alloca — size_bytes is set by the frontend for
                // address-taken allocas that survive mem2reg. When mem2reg
                // promotes an alloca, `size_bytes` may be `None`; fall back
                // to the pre-computed `alloca_size_map` before using TOP.
                if let Operation::Alloca { size_bytes } = &inst.op {
                    if let Some(dst) = inst.dst {
                        let size_interval = if let Some(size) = size_bytes {
                            Interval::singleton(i128::from(*size), 64)
                        } else if let Some(cached) = alloca_size_map.get(&dst) {
                            cached.clone()
                        } else {
                            Interval::make_top(64)
                        };
                        sizes.insert(dst, size_interval);
                    }
                }
            }
        }
    }

    sizes
}

/// Build a map from (`LocId`, byte_offset) to remaining buffer size for struct fields.
///
/// For each allocation whose type is a struct (identified via the `obj_type_map`),
/// computes the available buffer size from each field's byte offset to the end
/// of the struct. This enables the checker to compare memcpy sizes against the
/// space actually available from a struct field pointer, rather than the total
/// allocation size.
///
/// Only produces entries when PTA provides location info and the type registry
/// has struct field layout information.
fn build_field_allocation_sizes(
    module: &AirModule,
    pta: &PtaIntegration<'_>,
    obj_type_map: &BTreeMap<ObjId, TypeId>,
) -> BTreeMap<(LocId, u64), Interval> {
    let mut field_sizes = BTreeMap::new();

    let Some(pta_ref) = pta.pta_ref() else {
        return field_sizes;
    };

    // For each allocation with a known struct type, compute field remaining sizes
    for (obj_id, type_id) in obj_type_map {
        let Some(AirType::Struct { fields, total_size }) = module.types.get(type_id) else {
            continue;
        };

        // Find all LocIds for this object
        for (loc_id, location) in pta_ref.locations() {
            if location.obj != *obj_id {
                continue;
            }

            // For each field, store the remaining size from that offset to end of struct.
            // This is the relevant size for buffer overflow detection:
            // memcpy(field_ptr, src, N) overflows if N > (total_size - byte_offset).
            for field in fields {
                if let Some(byte_offset) = field.byte_offset {
                    let remaining = total_size.saturating_sub(byte_offset);
                    if remaining > 0 {
                        field_sizes.insert(
                            (*loc_id, byte_offset),
                            Interval::singleton(i128::from(remaining), 64),
                        );
                    }
                }
            }
        }
    }

    field_sizes
}

#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::air::{AirParam, CastKind, FieldPath};
    use saf_core::ids::{BlockId, InstId, ModuleId};

    fn vid(n: u128) -> ValueId {
        ValueId::new(n)
    }

    fn iid(n: u128) -> InstId {
        InstId::new(n)
    }

    /// Guard level for the bounds-check test module.
    #[derive(Clone, Copy)]
    enum BoundsGuard {
        /// Both lower (`i >= 0`) and upper (`i < 10`) checks present.
        Full,
        /// Only lower bound (`i >= 0`) check; upper bound unconstrained.
        LowerOnly,
        /// No bounds checks; entry branches directly to the access block.
        None,
    }

    /// Build an AIR module modeling a bounds-checked array access:
    ///
    /// ```text
    /// func test_narrowing(%i: i32):
    ///   entry:
    ///     %buf = alloca [10 x i32]  (size_bytes = 40)
    ///     %cmp1 = icmp sge %i, 0       // only if lower guard
    ///     br %cmp1, check2, exit        // only if lower guard
    ///   check2:
    ///     %cmp2 = icmp slt %i, 10      // only if Full guard
    ///     br %cmp2, access, exit        // only if Full guard
    ///   access:
    ///     %idx = sext i32 %i to i64
    ///     %gep = getelementptr [10 x i32], ptr %buf, i64 0, i64 %idx
    ///     ret
    ///   exit:
    ///     ret
    /// ```
    ///
    /// `guard` controls which bounds checks are included:
    /// - `Full`: both `i >= 0` and `i < 10` (index narrowed to `[0, 9]`)
    /// - `LowerOnly`: only `i >= 0` (index narrowed to `[0, +inf)`)
    /// - `None`: no checks, entry branches directly to access
    fn make_bounds_check_module(guard: BoundsGuard) -> AirModule {
        let mut module = AirModule::new(ModuleId::derive(b"test_narrowing"));

        // Constants: 0 and 10
        module
            .constants
            .insert(vid(3), Constant::Int { value: 0, bits: 32 });
        module.constants.insert(
            vid(4),
            Constant::Int {
                value: 10,
                bits: 32,
            },
        );

        let func_id = FunctionId::new(1);
        let mut func = AirFunction::new(func_id, "test_narrowing");
        func.params = vec![AirParam::new(vid(2), 0)];

        let entry_id = BlockId::new(1);
        let check2_id = BlockId::new(2);
        let access_id = BlockId::new(3);
        let exit_id = BlockId::new(4);

        let has_lower = matches!(guard, BoundsGuard::Full | BoundsGuard::LowerOnly);
        let has_upper = matches!(guard, BoundsGuard::Full);

        // --- entry block ---
        let mut entry = AirBlock::with_label(entry_id, "entry");

        // %buf = alloca [10 x i32], size_bytes = 40
        entry.instructions.push(
            Instruction::new(
                iid(10),
                Operation::Alloca {
                    size_bytes: Some(40),
                },
            )
            .with_dst(vid(1)),
        );

        if has_lower {
            // %cmp1 = icmp sge %i, 0
            entry.instructions.push(
                Instruction::new(
                    iid(20),
                    Operation::BinaryOp {
                        kind: BinaryOp::ICmpSge,
                    },
                )
                .with_operands(vec![vid(2), vid(3)])
                .with_dst(vid(5)),
            );
            // br %cmp1, check2, exit
            entry.instructions.push(
                Instruction::new(
                    iid(30),
                    Operation::CondBr {
                        then_target: check2_id,
                        else_target: exit_id,
                    },
                )
                .with_operands(vec![vid(5)]),
            );
        } else {
            // No bounds check — unconditional branch to access
            entry.instructions.push(Instruction::new(
                iid(30),
                Operation::Br { target: access_id },
            ));
        }

        // --- check2 block ---
        let mut check2 = AirBlock::with_label(check2_id, "check2");
        if has_upper {
            // %cmp2 = icmp slt %i, 10
            check2.instructions.push(
                Instruction::new(
                    iid(40),
                    Operation::BinaryOp {
                        kind: BinaryOp::ICmpSlt,
                    },
                )
                .with_operands(vec![vid(2), vid(4)])
                .with_dst(vid(6)),
            );
            // br %cmp2, access, exit
            check2.instructions.push(
                Instruction::new(
                    iid(50),
                    Operation::CondBr {
                        then_target: access_id,
                        else_target: exit_id,
                    },
                )
                .with_operands(vec![vid(6)]),
            );
        } else if has_lower {
            // Lower-only: check2 reached from lower bound pass, go straight to access
            check2.instructions.push(Instruction::new(
                iid(50),
                Operation::Br { target: access_id },
            ));
        } else {
            // No guards: check2 is unreachable (entry goes directly to access)
            check2
                .instructions
                .push(Instruction::new(iid(50), Operation::Unreachable));
        }

        // --- access block ---
        let mut access = AirBlock::with_label(access_id, "access");
        // %idx = sext i32 %i to i64
        access.instructions.push(
            Instruction::new(
                iid(60),
                Operation::Cast {
                    kind: CastKind::SExt,
                    target_bits: Some(64),
                },
            )
            .with_operands(vec![vid(2)])
            .with_dst(vid(7)),
        );
        // %gep = getelementptr [10 x i32], ptr %buf, i64 0, i64 %idx
        // operands: [base_ptr, index]
        access.instructions.push(
            Instruction::new(
                iid(70),
                Operation::Gep {
                    field_path: FieldPath::empty(),
                },
            )
            .with_operands(vec![vid(1), vid(7)])
            .with_dst(vid(8)),
        );
        // ret
        access
            .instructions
            .push(Instruction::new(iid(80), Operation::Ret));

        // --- exit block ---
        let mut exit = AirBlock::with_label(exit_id, "exit");
        exit.instructions
            .push(Instruction::new(iid(90), Operation::Ret));

        func.add_block(entry);
        func.add_block(check2);
        func.add_block(access);
        func.add_block(exit);

        module.add_function(func);
        module
    }

    #[test]
    fn bounds_check_narrows_gep_index() {
        // With both bounds checks: i >= 0 && i < 10, narrowing should prove
        // the GEP index is in [0, 9] — no buffer overflow findings.
        let module = make_bounds_check_module(BoundsGuard::Full);
        let config = AbstractInterpConfig::default();
        let result = check_buffer_overflow(&module, &config);

        assert!(
            result.findings.is_empty(),
            "Expected zero findings with full bounds checks, got {}: {:?}",
            result.findings.len(),
            result
                .findings
                .iter()
                .map(|f| &f.description)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn no_bounds_check_produces_warning() {
        // Without any bounds checks: parameter %i is unconstrained (TOP),
        // the GEP index should trigger a buffer overflow warning.
        let module = make_bounds_check_module(BoundsGuard::None);
        let config = AbstractInterpConfig::default();
        let result = check_buffer_overflow(&module, &config);

        assert!(
            !result.findings.is_empty(),
            "Expected buffer overflow findings without bounds checks"
        );
        for finding in &result.findings {
            assert_eq!(finding.checker, NumericCheckerKind::BufferOverflow);
        }
    }

    #[test]
    fn lower_bound_only_still_warns() {
        // Only lower bound check (i >= 0): the upper bound is unconstrained,
        // so the index could exceed the allocation size — should warn.
        let module = make_bounds_check_module(BoundsGuard::LowerOnly);
        let config = AbstractInterpConfig::default();
        let result = check_buffer_overflow(&module, &config);

        assert!(
            !result.findings.is_empty(),
            "Expected buffer overflow warning with only lower-bound check"
        );
        for finding in &result.findings {
            assert_eq!(finding.checker, NumericCheckerKind::BufferOverflow);
        }
        // Should be Warning (not Error) since the index *may* exceed bounds
        // but is not guaranteed to (lower half [0, ..] is non-negative).
        assert!(
            result
                .findings
                .iter()
                .any(|f| f.severity == NumericSeverity::Warning),
            "Expected at least one Warning severity for partial bounds check"
        );
    }

    #[test]
    fn severity_names() {
        assert_eq!(NumericSeverity::Safe.name(), "safe");
        assert_eq!(NumericSeverity::Warning.name(), "warning");
        assert_eq!(NumericSeverity::Error.name(), "error");
    }

    #[test]
    fn checker_kind_cwe() {
        assert_eq!(NumericCheckerKind::BufferOverflow.cwe(), 120);
        assert_eq!(NumericCheckerKind::IntegerOverflow.cwe(), 190);
        assert_eq!(NumericCheckerKind::DivisionByZero.cwe(), 369);
        assert_eq!(NumericCheckerKind::ShiftCount.cwe(), 682);
    }

    #[test]
    fn checker_kind_names() {
        assert_eq!(NumericCheckerKind::BufferOverflow.name(), "buffer_overflow");
        assert_eq!(
            NumericCheckerKind::IntegerOverflow.name(),
            "integer_overflow"
        );
        assert_eq!(
            NumericCheckerKind::DivisionByZero.name(),
            "division_by_zero"
        );
        assert_eq!(NumericCheckerKind::ShiftCount.name(), "shift_count");
    }

    #[test]
    fn empty_module_no_findings() {
        let module = AirModule::new(ModuleId::derive(b"test"));
        let config = AbstractInterpConfig::default();

        let result = check_buffer_overflow(&module, &config);
        assert!(result.findings.is_empty());

        let result = check_integer_overflow(&module, &config);
        assert!(result.findings.is_empty());

        let result = check_division_by_zero(&module, &config);
        assert!(result.findings.is_empty());

        let result = check_shift_count(&module, &config);
        assert!(result.findings.is_empty());
    }

    #[test]
    fn check_all_empty() {
        let module = AirModule::new(ModuleId::derive(b"test"));
        let config = AbstractInterpConfig::default();

        let result = check_all_numeric(&module, &config);
        assert!(result.findings.is_empty());
    }
}
