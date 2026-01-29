//! Typestate analysis via the IDE framework.
//!
//! Tracks per-resource state machines (e.g., file: `Uninit -> Opened -> Closed`)
//! using the IDE solver. Resources are tracked as facts, states as lattice values.
//!
//! Provides:
//! - [`TypestateSpec`] — declarative state machine specification
//! - [`TypestateLattice`] — finite lattice from state enum
//! - [`TypestateIdeProblem`] — IDE problem implementation
//! - Built-in specs: `file_io`, `mutex_lock`, `memory_alloc`

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use saf_core::air::{AirFunction, AirModule, Instruction, Operation};
use saf_core::ids::{BlockId, FunctionId, ValueId};

use super::edge_fn::BuiltinEdgeFn;
use super::ide_problem::IdeProblem;
use super::lattice::Lattice;
use super::matches_name;
use super::problem::IfdsProblem;

// ── TypestateSpec ────────────────────────────────────────────────────────

/// A declarative typestate specification.
///
/// Defines a state machine with named states, transitions triggered by
/// function calls, error states, and accepting states.
#[derive(Debug, Clone)]
pub struct TypestateSpec {
    /// Name of the typestate checker (e.g., "file_io").
    pub name: String,
    /// All possible states.
    pub states: Vec<String>,
    /// The initial state (set when a resource is created via a constructor).
    pub initial_state: String,
    /// States indicating a bug (e.g., "error").
    pub error_states: Vec<String>,
    /// States that are acceptable at program exit (e.g., "closed", "uninit").
    pub accepting_states: Vec<String>,
    /// State transitions triggered by function calls.
    pub transitions: Vec<TypestateTransition>,
    /// Functions that create resources (e.g., "fopen", "malloc").
    pub constructors: Vec<String>,
}

/// A single state transition in a typestate specification.
#[derive(Debug, Clone)]
pub struct TypestateTransition {
    /// Source state.
    pub from: String,
    /// Function name (glob pattern) that triggers the transition.
    pub call: String,
    /// Target state.
    pub to: String,
}

impl TypestateSpec {
    /// Validate this spec: check for duplicate states, valid references.
    pub fn validate(&self) -> Result<(), String> {
        // Check no duplicate states.
        let mut seen = BTreeSet::new();
        for s in &self.states {
            if !seen.insert(s.clone()) {
                return Err(format!("duplicate state: {s}"));
            }
        }
        // Check initial_state is in states.
        if !seen.contains(&self.initial_state) {
            return Err(format!(
                "initial_state '{}' not in states",
                self.initial_state
            ));
        }
        // Check error_states are in states.
        for e in &self.error_states {
            if !seen.contains(e) {
                return Err(format!("error_state '{e}' not in states"));
            }
        }
        // Check accepting_states are in states.
        for a in &self.accepting_states {
            if !seen.contains(a) {
                return Err(format!("accepting_state '{a}' not in states"));
            }
        }
        // Check transitions reference valid states.
        for t in &self.transitions {
            if !seen.contains(&t.from) {
                return Err(format!("transition from '{}' not in states", t.from));
            }
            if !seen.contains(&t.to) {
                return Err(format!("transition to '{}' not in states", t.to));
            }
        }
        Ok(())
    }

    /// Get the state index for a state name.
    fn state_index(&self, name: &str) -> Option<usize> {
        self.states.iter().position(|s| s == name)
    }

    /// Get the transition target state for a given (from_state, function_name).
    #[must_use]
    pub fn transition_for(&self, from_state: usize, func_name: &str) -> Option<usize> {
        for t in &self.transitions {
            let from_idx = self.state_index(&t.from)?;
            if from_idx == from_state && matches_name(func_name, &t.call) {
                return self.state_index(&t.to);
            }
        }
        None
    }

    /// Check if a function name matches any constructor pattern.
    fn is_constructor(&self, func_name: &str) -> bool {
        self.constructors.iter().any(|c| matches_name(func_name, c))
    }

    /// Check if a function name matches any transition trigger.
    fn is_transition_trigger(&self, func_name: &str) -> bool {
        self.transitions
            .iter()
            .any(|t| matches_name(func_name, &t.call))
    }

    /// Check if a state index is an error state.
    fn is_error_state(&self, idx: usize) -> bool {
        if let Some(name) = self.states.get(idx) {
            self.error_states.contains(name)
        } else {
            false
        }
    }

    /// Check if a state index is an accepting state.
    fn is_accepting_state(&self, idx: usize) -> bool {
        if let Some(name) = self.states.get(idx) {
            self.accepting_states.contains(name)
        } else {
            false
        }
    }
}

/// Get a built-in typestate spec by name.
///
/// Available specs: `"file_io"`, `"mutex_lock"`, `"memory_alloc"`.
#[must_use]
pub fn builtin_typestate_spec(name: &str) -> Option<TypestateSpec> {
    match name {
        "file_io" => Some(file_io_spec()),
        "mutex_lock" => Some(mutex_lock_spec()),
        "memory_alloc" => Some(memory_alloc_spec()),
        _ => None,
    }
}

/// File I/O typestate spec (matches PhASAR's `CSTDFILEIOTypeStateDescription`).
// NOTE: This spec is a declarative data structure defining the file I/O state machine.
// The function is long because it fully specifies transitions for 14 I/O operations
// across 4 states. Extracting sub-functions would obscure the spec's cohesive structure.
#[allow(clippy::too_many_lines)]
fn file_io_spec() -> TypestateSpec {
    TypestateSpec {
        name: "file_io".to_string(),
        states: vec![
            "uninit".to_string(),
            "opened".to_string(),
            "closed".to_string(),
            "error".to_string(),
        ],
        initial_state: "opened".to_string(),
        error_states: vec!["error".to_string()],
        accepting_states: vec!["uninit".to_string(), "closed".to_string()],
        transitions: vec![
            // opened + IO ops -> opened (stay)
            TypestateTransition {
                from: "opened".to_string(),
                call: "fread".to_string(),
                to: "opened".to_string(),
            },
            TypestateTransition {
                from: "opened".to_string(),
                call: "fwrite".to_string(),
                to: "opened".to_string(),
            },
            TypestateTransition {
                from: "opened".to_string(),
                call: "fgetc".to_string(),
                to: "opened".to_string(),
            },
            TypestateTransition {
                from: "opened".to_string(),
                call: "fgets".to_string(),
                to: "opened".to_string(),
            },
            TypestateTransition {
                from: "opened".to_string(),
                call: "fputc".to_string(),
                to: "opened".to_string(),
            },
            TypestateTransition {
                from: "opened".to_string(),
                call: "fputs".to_string(),
                to: "opened".to_string(),
            },
            TypestateTransition {
                from: "opened".to_string(),
                call: "fprintf".to_string(),
                to: "opened".to_string(),
            },
            TypestateTransition {
                from: "opened".to_string(),
                call: "fscanf".to_string(),
                to: "opened".to_string(),
            },
            TypestateTransition {
                from: "opened".to_string(),
                call: "fflush".to_string(),
                to: "opened".to_string(),
            },
            TypestateTransition {
                from: "opened".to_string(),
                call: "fseek".to_string(),
                to: "opened".to_string(),
            },
            TypestateTransition {
                from: "opened".to_string(),
                call: "ftell".to_string(),
                to: "opened".to_string(),
            },
            TypestateTransition {
                from: "opened".to_string(),
                call: "rewind".to_string(),
                to: "opened".to_string(),
            },
            TypestateTransition {
                from: "opened".to_string(),
                call: "feof".to_string(),
                to: "opened".to_string(),
            },
            TypestateTransition {
                from: "opened".to_string(),
                call: "ferror".to_string(),
                to: "opened".to_string(),
            },
            // opened + fclose -> closed
            TypestateTransition {
                from: "opened".to_string(),
                call: "fclose".to_string(),
                to: "closed".to_string(),
            },
            // closed + fclose -> error (double-close)
            TypestateTransition {
                from: "closed".to_string(),
                call: "fclose".to_string(),
                to: "error".to_string(),
            },
            // closed + IO -> error (use-after-close)
            TypestateTransition {
                from: "closed".to_string(),
                call: "fread".to_string(),
                to: "error".to_string(),
            },
            TypestateTransition {
                from: "closed".to_string(),
                call: "fwrite".to_string(),
                to: "error".to_string(),
            },
            TypestateTransition {
                from: "closed".to_string(),
                call: "fgetc".to_string(),
                to: "error".to_string(),
            },
            TypestateTransition {
                from: "closed".to_string(),
                call: "fgets".to_string(),
                to: "error".to_string(),
            },
            TypestateTransition {
                from: "closed".to_string(),
                call: "fputs".to_string(),
                to: "error".to_string(),
            },
            TypestateTransition {
                from: "closed".to_string(),
                call: "fprintf".to_string(),
                to: "error".to_string(),
            },
            // uninit + fclose -> error (close-before-open)
            TypestateTransition {
                from: "uninit".to_string(),
                call: "fclose".to_string(),
                to: "error".to_string(),
            },
            // uninit + IO -> error (use-before-open)
            TypestateTransition {
                from: "uninit".to_string(),
                call: "fread".to_string(),
                to: "error".to_string(),
            },
            TypestateTransition {
                from: "uninit".to_string(),
                call: "fwrite".to_string(),
                to: "error".to_string(),
            },
        ],
        constructors: vec!["fopen".to_string(), "fdopen".to_string()],
    }
}

/// Mutex lock typestate spec.
fn mutex_lock_spec() -> TypestateSpec {
    TypestateSpec {
        name: "mutex_lock".to_string(),
        states: vec![
            "uninit".to_string(),
            "unlocked".to_string(),
            "locked".to_string(),
            "error".to_string(),
        ],
        initial_state: "unlocked".to_string(),
        error_states: vec!["error".to_string()],
        accepting_states: vec!["uninit".to_string(), "unlocked".to_string()],
        transitions: vec![
            TypestateTransition {
                from: "unlocked".to_string(),
                call: "pthread_mutex_lock".to_string(),
                to: "locked".to_string(),
            },
            TypestateTransition {
                from: "locked".to_string(),
                call: "pthread_mutex_unlock".to_string(),
                to: "unlocked".to_string(),
            },
            TypestateTransition {
                from: "locked".to_string(),
                call: "pthread_mutex_lock".to_string(),
                to: "error".to_string(),
            },
            TypestateTransition {
                from: "unlocked".to_string(),
                call: "pthread_mutex_unlock".to_string(),
                to: "error".to_string(),
            },
        ],
        constructors: vec!["pthread_mutex_init".to_string()],
    }
}

/// Memory allocation typestate spec.
fn memory_alloc_spec() -> TypestateSpec {
    TypestateSpec {
        name: "memory_alloc".to_string(),
        states: vec![
            "unallocated".to_string(),
            "allocated".to_string(),
            "freed".to_string(),
            "error".to_string(),
        ],
        initial_state: "allocated".to_string(),
        error_states: vec!["error".to_string()],
        accepting_states: vec!["unallocated".to_string(), "freed".to_string()],
        transitions: vec![
            TypestateTransition {
                from: "allocated".to_string(),
                call: "free".to_string(),
                to: "freed".to_string(),
            },
            TypestateTransition {
                from: "freed".to_string(),
                call: "free".to_string(),
                to: "error".to_string(),
            },
            TypestateTransition {
                from: "unallocated".to_string(),
                call: "free".to_string(),
                to: "error".to_string(),
            },
        ],
        constructors: vec![
            "malloc".to_string(),
            "calloc".to_string(),
            "realloc".to_string(),
        ],
    }
}

// ── TypestateLattice ─────────────────────────────────────────────────────

/// A finite lattice derived from a typestate spec's states.
///
/// Ordering: `Top > State(0) | State(1) | ... | State(n) > Bottom`
///
/// Join of different concrete states = Top (over-approximation).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TypestateLattice {
    /// Bottom (unused / contradictory).
    Bottom,
    /// A concrete state (index into `TypestateSpec.states`).
    State(usize),
    /// Top (unknown / all possible states).
    Top,
}

impl Lattice for TypestateLattice {
    fn top() -> Self {
        TypestateLattice::Top
    }

    fn bottom() -> Self {
        TypestateLattice::Bottom
    }

    fn join(&self, other: &Self) -> Self {
        if self == other {
            return self.clone();
        }
        match (self, other) {
            (TypestateLattice::Bottom, x) | (x, TypestateLattice::Bottom) => x.clone(),
            // Top dominates; two different concrete states also join to Top (over-approximation)
            _ => TypestateLattice::Top,
        }
    }

    fn meet(&self, other: &Self) -> Self {
        if self == other {
            return self.clone();
        }
        match (self, other) {
            (TypestateLattice::Top, x) | (x, TypestateLattice::Top) => x.clone(),
            // Bottom absorbs
            (TypestateLattice::Bottom, _) | (_, TypestateLattice::Bottom) => {
                TypestateLattice::Bottom
            }
            // Two different concrete states meet to Bottom (under-approximation)
            _ => TypestateLattice::Bottom,
        }
    }

    fn leq(&self, other: &Self) -> bool {
        self.join(other) == *other
    }
}

impl fmt::Display for TypestateLattice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypestateLattice::Bottom => write!(f, "bottom"),
            TypestateLattice::State(idx) => write!(f, "state({idx})"),
            TypestateLattice::Top => write!(f, "top"),
        }
    }
}

// ── TypestateFact ────────────────────────────────────────────────────────

/// Data-flow fact for typestate analysis.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TypestateFact {
    /// The zero (tautology) fact.
    Zero,
    /// A tracked resource identified by its `ValueId`.
    Tracked(ValueId),
}

// ── TypestateIdeProblem ──────────────────────────────────────────────────

/// Typestate analysis as an IDE problem.
///
/// - **Fact** = `TypestateFact::Zero | TypestateFact::Tracked(ValueId)`
/// - **Value** = `TypestateLattice` (which state the resource is in)
/// - **Edge functions** = state transitions from the spec
pub struct TypestateIdeProblem<'a> {
    module: &'a AirModule,
    spec: TypestateSpec,
    /// Functions whose calls create resources (constructors).
    constructor_funcs: BTreeSet<FunctionId>,
    /// Functions whose calls trigger transitions.
    transition_funcs: BTreeSet<FunctionId>,
    /// Function name lookup (FunctionId -> name).
    func_names: BTreeMap<FunctionId, String>,
}

impl<'a> TypestateIdeProblem<'a> {
    /// Create a new typestate IDE problem.
    pub fn new(module: &'a AirModule, spec: TypestateSpec) -> Self {
        let mut constructor_funcs = BTreeSet::new();
        let mut transition_funcs = BTreeSet::new();
        let mut func_names = BTreeMap::new();

        for func in &module.functions {
            func_names.insert(func.id, func.name.clone());
            if spec.is_constructor(&func.name) {
                constructor_funcs.insert(func.id);
            }
            if spec.is_transition_trigger(&func.name) {
                transition_funcs.insert(func.id);
            }
        }

        Self {
            module,
            spec,
            constructor_funcs,
            transition_funcs,
            func_names,
        }
    }

    /// Check if a function is a constructor.
    fn is_constructor(&self, func_id: FunctionId) -> bool {
        self.constructor_funcs.contains(&func_id)
    }

    /// Check if a function triggers a transition.
    fn is_transition(&self, func_id: FunctionId) -> bool {
        self.transition_funcs.contains(&func_id)
    }

    /// Get function name.
    fn func_name(&self, func_id: FunctionId) -> Option<&str> {
        self.func_names.get(&func_id).map(String::as_str)
    }

    /// Get the initial state index.
    fn initial_state_idx(&self) -> usize {
        self.spec.state_index(&self.spec.initial_state).unwrap_or(0)
    }

    /// Collect all findings from the IDE result.
    ///
    /// A finding is generated for:
    /// - Resources in error states at any point
    /// - Resources in non-accepting states at function exits
    pub fn collect_findings(
        &self,
        result: &super::ide_result::IdeResult<TypestateFact, TypestateLattice>,
    ) -> Vec<TypestateFinding> {
        let mut findings = Vec::new();

        // Build instruction → (function, block) lookup.
        let mut inst_location: BTreeMap<saf_core::ids::InstId, (FunctionId, BlockId)> =
            BTreeMap::new();
        for func in &self.module.functions {
            if func.is_declaration {
                continue;
            }
            for block in &func.blocks {
                for inst in &block.instructions {
                    inst_location.insert(inst.id, (func.id, block.id));
                }
            }
        }

        for (inst_id, fact_map) in &result.values {
            for (fact, value) in fact_map {
                if let TypestateFact::Tracked(vid) = fact {
                    if let TypestateLattice::State(idx) = value {
                        if self.spec.is_error_state(*idx) {
                            let state_name = self
                                .spec
                                .states
                                .get(*idx)
                                .cloned()
                                .unwrap_or_else(|| format!("state({idx})"));
                            let location = inst_location.get(inst_id).copied();
                            findings.push(TypestateFinding {
                                resource: *vid,
                                state: state_name,
                                inst: *inst_id,
                                kind: TypestateFindingKind::ErrorState,
                                spec_name: self.spec.name.clone(),
                                location,
                            });
                        }
                    }
                }
            }
        }

        // Check for non-accepting states at function exits.
        for func in &self.module.functions {
            if func.is_declaration {
                continue;
            }
            for block in &func.blocks {
                if let Some(term) = block.terminator() {
                    if matches!(term.op, Operation::Ret) {
                        if let Some(fact_map) = result.values.get(&term.id) {
                            for (fact, value) in fact_map {
                                if let TypestateFact::Tracked(vid) = fact {
                                    if let TypestateLattice::State(idx) = value {
                                        if !self.spec.is_accepting_state(*idx)
                                            && !self.spec.is_error_state(*idx)
                                        {
                                            let state_name = self
                                                .spec
                                                .states
                                                .get(*idx)
                                                .cloned()
                                                .unwrap_or_else(|| format!("state({idx})"));
                                            findings.push(TypestateFinding {
                                                resource: *vid,
                                                state: state_name,
                                                inst: term.id,
                                                kind: TypestateFindingKind::NonAcceptingAtExit,
                                                spec_name: self.spec.name.clone(),
                                                location: Some((func.id, block.id)),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Deduplicate by (resource, kind).
        findings.sort();
        findings.dedup();
        findings
    }
}

/// A typestate analysis finding.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TypestateFinding {
    /// The resource `ValueId`.
    pub resource: ValueId,
    /// The state the resource is in.
    pub state: String,
    /// The instruction where the finding was detected.
    pub inst: saf_core::ids::InstId,
    /// The kind of finding.
    pub kind: TypestateFindingKind,
    /// Which spec produced this finding.
    pub spec_name: String,
    /// Location of the finding: `(FunctionId, BlockId)`, or `None` if the
    /// instruction could not be mapped to a function/block.
    pub location: Option<(saf_core::ids::FunctionId, saf_core::ids::BlockId)>,
}

/// Kind of typestate finding.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TypestateFindingKind {
    /// Resource reached an error state (e.g., double-close).
    ErrorState,
    /// Resource not in an accepting state at function exit (e.g., file leak).
    NonAcceptingAtExit,
}

impl fmt::Display for TypestateFindingKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypestateFindingKind::ErrorState => write!(f, "error_state"),
            TypestateFindingKind::NonAcceptingAtExit => write!(f, "non_accepting_at_exit"),
        }
    }
}

// ── IfdsProblem implementation ───────────────────────────────────────────

impl IfdsProblem for TypestateIdeProblem<'_> {
    type Fact = TypestateFact;

    fn zero_value(&self) -> Self::Fact {
        TypestateFact::Zero
    }

    fn module(&self) -> &AirModule {
        self.module
    }

    fn initial_seeds(&self) -> BTreeMap<FunctionId, BTreeSet<Self::Fact>> {
        BTreeMap::new()
    }

    fn normal_flow(&self, inst: &Instruction, fact: &Self::Fact) -> BTreeSet<Self::Fact> {
        match &inst.op {
            // Constructor call: zero generates Tracked(dst).
            Operation::CallDirect { callee } if self.is_constructor(*callee) => {
                if let TypestateFact::Zero = fact {
                    let mut result: BTreeSet<Self::Fact> = [fact.clone()].into_iter().collect();
                    if let Some(dst) = inst.dst {
                        result.insert(TypestateFact::Tracked(dst));
                    }
                    return result;
                }
                // Tracked facts pass through (their value will be updated by edge_fn).
                [fact.clone()].into_iter().collect()
            }
            // HeapAlloc operations (malloc, calloc, etc.) are also constructors for memory_alloc spec.
            Operation::HeapAlloc { kind } if self.spec.is_constructor(kind.as_str()) => {
                if let TypestateFact::Zero = fact {
                    let mut result: BTreeSet<Self::Fact> = [fact.clone()].into_iter().collect();
                    if let Some(dst) = inst.dst {
                        result.insert(TypestateFact::Tracked(dst));
                    }
                    return result;
                }
                [fact.clone()].into_iter().collect()
            }
            // Transition-triggering call: keep tracked facts (edge_fn handles value).
            Operation::CallDirect { callee } if self.is_transition(*callee) => {
                // For tracked resources: if the resource is passed as first argument,
                // keep the fact (edge_fn will transition the value).
                if let TypestateFact::Tracked(v) = fact {
                    if !inst.operands.is_empty() && inst.operands[0] == *v {
                        return [fact.clone()].into_iter().collect();
                    }
                }
                [fact.clone()].into_iter().collect()
            }
            // Store: propagate tracked fact to the stored-to pointer.
            Operation::Store => {
                if let TypestateFact::Tracked(v) = fact {
                    if inst.operands.len() >= 2 && inst.operands[0] == *v {
                        let mut result: BTreeSet<Self::Fact> = [fact.clone()].into_iter().collect();
                        result.insert(TypestateFact::Tracked(inst.operands[1]));
                        return result;
                    }
                }
                [fact.clone()].into_iter().collect()
            }
            // Copy/Cast/Load/GEP: propagate tracking.
            Operation::Copy
            | Operation::Freeze
            | Operation::Cast { .. }
            | Operation::Load
            | Operation::Gep { .. } => {
                if let TypestateFact::Tracked(v) = fact {
                    if inst.operands.contains(v) {
                        let mut result: BTreeSet<Self::Fact> = [fact.clone()].into_iter().collect();
                        if let Some(dst) = inst.dst {
                            result.insert(TypestateFact::Tracked(dst));
                        }
                        return result;
                    }
                }
                [fact.clone()].into_iter().collect()
            }
            // Phi: propagate tracking from incoming values.
            Operation::Phi { incoming } => {
                if let TypestateFact::Tracked(v) = fact {
                    for (_, val) in incoming {
                        if val == v {
                            let mut result: BTreeSet<Self::Fact> =
                                [fact.clone()].into_iter().collect();
                            if let Some(dst) = inst.dst {
                                result.insert(TypestateFact::Tracked(dst));
                            }
                            return result;
                        }
                    }
                }
                [fact.clone()].into_iter().collect()
            }
            _ => [fact.clone()].into_iter().collect(),
        }
    }

    fn call_flow(
        &self,
        call_site: &Instruction,
        callee: &AirFunction,
        fact: &Self::Fact,
    ) -> BTreeSet<Self::Fact> {
        match fact {
            TypestateFact::Zero => [TypestateFact::Zero].into_iter().collect(),
            TypestateFact::Tracked(v) => {
                let mut result = BTreeSet::new();
                for (i, operand) in call_site.operands.iter().enumerate() {
                    if operand == v {
                        if let Some(param) = callee.params.get(i) {
                            result.insert(TypestateFact::Tracked(param.id));
                        }
                    }
                }
                result
            }
        }
    }

    fn return_flow(
        &self,
        call_site: &Instruction,
        _callee: &AirFunction,
        exit_inst: &Instruction,
        fact: &Self::Fact,
    ) -> BTreeSet<Self::Fact> {
        match fact {
            TypestateFact::Zero => BTreeSet::new(),
            TypestateFact::Tracked(v) => {
                let mut result = BTreeSet::new();
                if exit_inst.operands.contains(v) {
                    if let Some(dst) = call_site.dst {
                        result.insert(TypestateFact::Tracked(dst));
                    }
                }
                result
            }
        }
    }

    fn call_to_return_flow(
        &self,
        call_site: &Instruction,
        fact: &Self::Fact,
    ) -> BTreeSet<Self::Fact> {
        match fact {
            TypestateFact::Zero => [TypestateFact::Zero].into_iter().collect(),
            TypestateFact::Tracked(v) => {
                if call_site.operands.contains(v) {
                    return BTreeSet::new();
                }
                [fact.clone()].into_iter().collect()
            }
        }
    }
}

// ── IdeProblem implementation ────────────────────────────────────────────

impl IdeProblem for TypestateIdeProblem<'_> {
    type Value = TypestateLattice;

    fn normal_edge_fn(
        &self,
        inst: &Instruction,
        src_fact: &Self::Fact,
        succ_fact: &Self::Fact,
    ) -> BuiltinEdgeFn<Self::Value> {
        match &inst.op {
            // Constructor: Zero -> Tracked(dst): assign initial state.
            Operation::CallDirect { callee } if self.is_constructor(*callee) => {
                if matches!(src_fact, TypestateFact::Zero)
                    && matches!(succ_fact, TypestateFact::Tracked(_))
                {
                    let initial_idx = self.initial_state_idx();
                    return BuiltinEdgeFn::Constant(TypestateLattice::State(initial_idx));
                }
                BuiltinEdgeFn::Identity
            }
            // HeapAlloc constructor: Zero -> Tracked(dst): assign initial state.
            Operation::HeapAlloc { kind } if self.spec.is_constructor(kind.as_str()) => {
                if matches!(src_fact, TypestateFact::Zero)
                    && matches!(succ_fact, TypestateFact::Tracked(_))
                {
                    let initial_idx = self.initial_state_idx();
                    return BuiltinEdgeFn::Constant(TypestateLattice::State(initial_idx));
                }
                BuiltinEdgeFn::Identity
            }
            // Transition call: Tracked(v) -> Tracked(v) with state transition.
            //
            // Apply the transition to ALL tracked facts that flow through,
            // not just the one matching operands[0]. In `-O0` LLVM IR, the
            // resource pointer is stored to an alloca and re-loaded before
            // each use, creating multiple SSA values that are all aliases of
            // the same resource. Transitioning only the direct argument would
            // leave alias facts in the old state, causing false positives.
            Operation::CallDirect { callee } if self.is_transition(*callee) => {
                if let TypestateFact::Tracked(_) = src_fact {
                    if src_fact == succ_fact {
                        if let Some(func_name) = self.func_name(*callee) {
                            let func_name = func_name.to_string();
                            return transition_edge_fn(&self.spec, &func_name);
                        }
                    }
                }
                BuiltinEdgeFn::Identity
            }
            _ => BuiltinEdgeFn::Identity,
        }
    }

    fn call_edge_fn(
        &self,
        _call_site: &Instruction,
        _callee: &AirFunction,
        _src_fact: &Self::Fact,
        _dest_fact: &Self::Fact,
    ) -> BuiltinEdgeFn<Self::Value> {
        BuiltinEdgeFn::Identity
    }

    fn return_edge_fn(
        &self,
        _call_site: &Instruction,
        _callee: &AirFunction,
        _exit_inst: &Instruction,
        _exit_fact: &Self::Fact,
        _ret_fact: &Self::Fact,
    ) -> BuiltinEdgeFn<Self::Value> {
        BuiltinEdgeFn::Identity
    }

    fn call_to_return_edge_fn(
        &self,
        _call_site: &Instruction,
        _src_fact: &Self::Fact,
        _ret_fact: &Self::Fact,
    ) -> BuiltinEdgeFn<Self::Value> {
        BuiltinEdgeFn::Identity
    }

    fn top_value(&self) -> Self::Value {
        TypestateLattice::Top
    }

    fn bottom_value(&self) -> Self::Value {
        TypestateLattice::Bottom
    }
}

/// Create a transition edge function for a given function call.
///
/// Builds a `TransitionTable` that maps the current state to the next state
/// according to the typestate spec's transitions for this function.
/// States without an explicit transition get self-loop entries to preserve
/// their value at CFG join points (where this table may meet `Identity`).
fn transition_edge_fn(spec: &TypestateSpec, func_name: &str) -> BuiltinEdgeFn<TypestateLattice> {
    let mut entries = Vec::new();
    for t in &spec.transitions {
        if matches_name(func_name, &t.call) {
            if let (Some(from_idx), Some(to_idx)) =
                (spec.state_index(&t.from), spec.state_index(&t.to))
            {
                entries.push((
                    TypestateLattice::State(from_idx),
                    TypestateLattice::State(to_idx),
                ));
            }
        }
    }

    if entries.is_empty() {
        return BuiltinEdgeFn::Identity;
    }

    // Add self-loop entries for states that have no transition for this function.
    // Without these, the `default: Top` would map untransitioned states to Top,
    // causing information loss when this table is joined with Identity at merge points.
    for (idx, _) in spec.states.iter().enumerate() {
        let state = TypestateLattice::State(idx);
        if !entries.iter().any(|(from, _)| *from == state) {
            entries.push((state.clone(), state));
        }
    }

    BuiltinEdgeFn::TransitionTable {
        entries,
        default: TypestateLattice::Top, // Truly unknown input -> unknown output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::callgraph::CallGraph;
    use crate::icfg::Icfg;
    use crate::ifds::config::IfdsConfig;
    use crate::ifds::ide_solver::solve_ide;
    use saf_core::air::{AirBlock, AirFunction};
    use saf_core::ids::{BlockId, InstId, ModuleId};

    fn make_module(functions: Vec<AirFunction>) -> AirModule {
        AirModule {
            id: ModuleId::derive(b"test_ts"),
            name: Some("test_ts".to_string()),
            functions,
            globals: Vec::new(),
            source_files: Vec::new(),
            type_hierarchy: Vec::new(),
            constants: std::collections::BTreeMap::new(),
            types: std::collections::BTreeMap::new(),
            target_pointer_width: 8,
            function_index: BTreeMap::new(),
            name_index: BTreeMap::new(),
        }
    }

    fn make_function(id: u128, name: &str, blocks: Vec<AirBlock>) -> AirFunction {
        AirFunction {
            id: FunctionId::new(id),
            name: name.to_string(),
            params: Vec::new(),
            blocks,
            entry_block: None,
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        }
    }

    fn make_declaration(id: u128, name: &str) -> AirFunction {
        AirFunction {
            id: FunctionId::new(id),
            name: name.to_string(),
            params: Vec::new(),
            blocks: Vec::new(),
            entry_block: None,
            is_declaration: true,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        }
    }

    fn make_block(id: u128, instructions: Vec<Instruction>) -> AirBlock {
        AirBlock {
            id: BlockId::new(id),
            label: None,
            instructions,
        }
    }

    // ── Spec tests ──────────────────────────────────────────────────────

    #[test]
    fn file_io_spec_validates() {
        let spec = file_io_spec();
        spec.validate().expect("file_io spec should be valid");
    }

    #[test]
    fn mutex_lock_spec_validates() {
        let spec = mutex_lock_spec();
        spec.validate().expect("mutex_lock spec should be valid");
    }

    #[test]
    fn memory_alloc_spec_validates() {
        let spec = memory_alloc_spec();
        spec.validate().expect("memory_alloc spec should be valid");
    }

    #[test]
    fn invalid_spec_detected() {
        let spec = TypestateSpec {
            name: "bad".to_string(),
            states: vec!["a".to_string()],
            initial_state: "b".to_string(), // not in states!
            error_states: vec![],
            accepting_states: vec![],
            transitions: vec![],
            constructors: vec![],
        };
        assert!(spec.validate().is_err());
    }

    #[test]
    fn builtin_spec_lookup() {
        assert!(builtin_typestate_spec("file_io").is_some());
        assert!(builtin_typestate_spec("mutex_lock").is_some());
        assert!(builtin_typestate_spec("memory_alloc").is_some());
        assert!(builtin_typestate_spec("nonexistent").is_none());
    }

    // ── Lattice tests ───────────────────────────────────────────────────

    #[test]
    fn typestate_lattice_top_bottom() {
        assert!(TypestateLattice::Bottom.leq(&TypestateLattice::Top));
        assert!(TypestateLattice::Bottom.leq(&TypestateLattice::State(0)));
        assert!(TypestateLattice::State(0).leq(&TypestateLattice::Top));
    }

    #[test]
    fn typestate_lattice_join_different_states() {
        let a = TypestateLattice::State(0);
        let b = TypestateLattice::State(1);
        assert_eq!(a.join(&b), TypestateLattice::Top);
    }

    #[test]
    fn typestate_lattice_join_same_state() {
        let a = TypestateLattice::State(0);
        assert_eq!(a.join(&a), TypestateLattice::State(0));
    }

    // ── IDE integration tests ───────────────────────────────────────────

    #[test]
    fn typestate_fopen_fclose_no_error() {
        // fopen() -> fclose(): should be in "closed" state (accepting).
        let fopen = make_declaration(2, "fopen");
        let fclose = make_declaration(3, "fclose");
        let module = make_module(vec![
            make_function(
                1,
                "main",
                vec![make_block(
                    1,
                    vec![
                        Instruction::new(
                            InstId::new(100),
                            Operation::CallDirect {
                                callee: FunctionId::new(2),
                            },
                        )
                        .with_dst(ValueId::new(10)),
                        Instruction::new(
                            InstId::new(101),
                            Operation::CallDirect {
                                callee: FunctionId::new(3),
                            },
                        )
                        .with_operands(vec![ValueId::new(10)]),
                        Instruction::new(InstId::new(102), Operation::Ret),
                    ],
                )],
            ),
            fopen,
            fclose,
        ]);
        let cg = CallGraph::build(&module);
        let icfg = Icfg::build(&module, &cg);
        let spec = file_io_spec();
        let problem = TypestateIdeProblem::new(&module, spec);
        let config = IfdsConfig::default();

        let result = solve_ide(&problem, &icfg, &cg, &config);

        // Resource should be tracked.
        assert!(result.holds_at(InstId::new(101), &TypestateFact::Tracked(ValueId::new(10))));

        // Collect findings — should have zero error findings for correct usage.
        let findings = problem.collect_findings(&result);
        let error_findings: Vec<_> = findings
            .iter()
            .filter(|f| f.kind == TypestateFindingKind::ErrorState)
            .collect();
        assert!(
            error_findings.is_empty(),
            "correct fopen/fclose should have no error findings, got: {error_findings:?}"
        );
    }

    #[test]
    fn typestate_fopen_no_fclose_leak() {
        // fopen() without fclose: resource in "opened" (non-accepting) at exit.
        let fopen = make_declaration(2, "fopen");
        let module = make_module(vec![
            make_function(
                1,
                "main",
                vec![make_block(
                    1,
                    vec![
                        Instruction::new(
                            InstId::new(100),
                            Operation::CallDirect {
                                callee: FunctionId::new(2),
                            },
                        )
                        .with_dst(ValueId::new(10)),
                        Instruction::new(InstId::new(101), Operation::Ret),
                    ],
                )],
            ),
            fopen,
        ]);
        let cg = CallGraph::build(&module);
        let icfg = Icfg::build(&module, &cg);
        let spec = file_io_spec();
        let problem = TypestateIdeProblem::new(&module, spec);
        let config = IfdsConfig::default();

        let result = solve_ide(&problem, &icfg, &cg, &config);

        // Resource is tracked.
        assert!(result.holds_at(InstId::new(101), &TypestateFact::Tracked(ValueId::new(10))));

        // Should have a non-accepting finding (leak).
        let findings = problem.collect_findings(&result);
        let leak_findings: Vec<_> = findings
            .iter()
            .filter(|f| f.kind == TypestateFindingKind::NonAcceptingAtExit)
            .collect();
        assert!(
            !leak_findings.is_empty(),
            "fopen without fclose should produce a non-accepting finding (leak)"
        );
    }

    #[test]
    fn typestate_double_close_error() {
        // fopen() -> fclose() -> fclose(): second fclose should produce error.
        let fopen = make_declaration(2, "fopen");
        let fclose = make_declaration(3, "fclose");
        let module = make_module(vec![
            make_function(
                1,
                "main",
                vec![make_block(
                    1,
                    vec![
                        Instruction::new(
                            InstId::new(100),
                            Operation::CallDirect {
                                callee: FunctionId::new(2),
                            },
                        )
                        .with_dst(ValueId::new(10)),
                        Instruction::new(
                            InstId::new(101),
                            Operation::CallDirect {
                                callee: FunctionId::new(3),
                            },
                        )
                        .with_operands(vec![ValueId::new(10)]),
                        Instruction::new(
                            InstId::new(102),
                            Operation::CallDirect {
                                callee: FunctionId::new(3),
                            },
                        )
                        .with_operands(vec![ValueId::new(10)]),
                        Instruction::new(InstId::new(103), Operation::Ret),
                    ],
                )],
            ),
            fopen,
            fclose,
        ]);
        let cg = CallGraph::build(&module);
        let icfg = Icfg::build(&module, &cg);
        let spec = file_io_spec();
        let problem = TypestateIdeProblem::new(&module, spec);
        let config = IfdsConfig::default();

        let result = solve_ide(&problem, &icfg, &cg, &config);

        // The fact should exist.
        assert!(result.holds_at(InstId::new(103), &TypestateFact::Tracked(ValueId::new(10))));

        // Should have an error finding (double-close).
        let findings = problem.collect_findings(&result);
        let error_findings: Vec<_> = findings
            .iter()
            .filter(|f| f.kind == TypestateFindingKind::ErrorState)
            .collect();
        assert!(
            !error_findings.is_empty(),
            "double fclose should produce an error finding, got findings: {findings:?}"
        );
    }
}
