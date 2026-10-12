//! Sound, fail-closed **bounded-exhaustive** symbolic proof of `unreach-call`
//! TRUE (plan 221 §3 B5).
//!
//! # Why a new engine
//!
//! The interval sentinel ([`crate::prove_unreachable`]) proves ~0.2 % of the
//! expected-TRUE pool: a non-relational domain cannot discharge the guards most
//! tasks hinge on (`x < y` from a loop condition, a bounded counter, an unsigned
//! comparison of a negative operand). The FALSE-side engines (`bmc`, `se_interp`)
//! reason precisely but are *deliberately* unsound (uniform 64-bit arithmetic,
//! identity casts, havoc) because their output is replay-gated.
//!
//! This engine is the sound counterpart: a forward symbolic executor from `main`
//! whose every modelled step is **exact machine semantics** — bitvectors of the
//! declared width, two's-complement arithmetic, exact `trunc`/`zext`/`sext`,
//! signed and unsigned comparisons — and which **abstains on anything it cannot
//! model exactly** (pointer arithmetic, stores through unknown pointers, floats,
//! indirect calls, recursion, unknown externs, vectors, big integers). Loops are
//! unwound by re-executing blocks; a loop that is not finished within the visit
//! cap, or an exploration that exceeds its step/state/solver/time budget, is an
//! abstain, never a proof.
//!
//! # The proof
//!
//! Every explored state carries its path condition. A conditional branch forks
//! only into successors whose path condition Z3 does not refute (an `UNSAT`
//! prune is exact; `UNKNOWN` is treated as feasible — over-approximation is the
//! safe direction). The property holds iff **every** path terminates (`main`
//! returns, `abort`/`exit`, `unreachable`, or a refuted assumption) without
//! executing a feasible `reach_error` / `__VERIFIER_error` call. If a feasible
//! error call is met the engine abstains — it never claims FALSE; that is the
//! replay-gated engines' job.
//!
//! Over-approximations used (each only ADDS behaviours, so a proof stays sound):
//! a load from memory the engine does not model (unknown pointer, escaped or
//! never-written cell, uninitialised alloca, extern global) is a fresh symbol;
//! `main`'s parameters are fresh; an allow-listed pure extern (`printf`, …)
//! returns a fresh value; a Z3 timeout counts as feasible. The one assumption
//! shared with the other engines: a division/remainder path is only explored
//! with `divisor != 0` (division by zero is undefined behaviour in C).
//!
//! # Not modelled (abstain)
//!
//! `Gep` results are opaque pointers and *escape* their base cell (later loads
//! from that cell are fresh); a `Store` through an opaque pointer abstains
//! because it could alias a modelled cell. `Memcpy`/`Memset`, `HeapAlloc`
//! stores, `CallIndirect`, non-scalar nondet, float/vector/pointer casts, and
//! any extern outside the allow-list abstain.

use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;
use std::time::{Duration, Instant};

use saf_core::air::{
    AirModule, AirType, BinaryOp, CastKind, Constant, GEP_BYTES_KEY, Instruction, Layout, Operation,
};
use saf_core::ids::{BlockId, FunctionId, InstId, ValueId};
use z3::SatResult;
use z3::ast::{Ast, BV, Bool};

use crate::property::{REACH_ERROR_NAMES, is_scalar_integer_nondet};

/// Max times one block may be (re-)entered on a single path (loop unwinding).
const VISIT_CAP: u32 = 4_096;
/// Max instructions executed across all states.
const MAX_STEPS: u64 = 3_000_000;
/// Max states dequeued.
const MAX_STATES: u64 = 200_000;
/// Max Z3 queries.
const MAX_SOLVER_CALLS: u64 = 60_000;
/// Max call-inlining depth.
const MAX_CALL_DEPTH: usize = 256;
/// Largest `k` tried by k-induction (plan 223).
const K_MAX: u32 = 5;
/// Most concrete states enumerated at one loop-header arrival before the
/// header fixpoint gives up on that header (its state is not finite enough).
const FIXPOINT_CAP: usize = 256;
/// Most concrete loop-header states kept in all: past this the header fixpoint
/// is switched off (a long counting loop is unrolled as before).
const FIXPOINT_TOTAL_CAP: usize = 50_000;
/// Per-query Z3 timeout (ms) for branch feasibility.
const Z3_TIMEOUT_MS: u32 = 2_000;
/// Per-query Z3 timeout (ms) for a VIOLATION check — the one query whose
/// UNSAT answer is the proof. Bit-blasted multiplication (`x * x` under a
/// `|x| < 46340` guard) regularly needs more than the branch budget, and an
/// UNKNOWN here costs the whole task; the check gets up to this much, capped
/// by the time the task has left.
const Z3_VIOLATION_TIMEOUT_MS: u32 = 120_000;
/// Deterministic Z3 budget / seed.
const Z3_RLIMIT: u32 = 20_000_000;
const Z3_SEED: u32 = 0;

/// Externs that terminate the program (the path ends; no error reached).
const TERMINATING: &[&str] = &[
    "abort",
    "exit",
    "_exit",
    "_Exit",
    "__assert_fail",
    "__assert_rtn",
    "llvm.trap",
    "__builtin_trap",
];
/// Externs with no effect on modelled state whose result (if any) is fresh.
/// Externs with no effect on the modelled state: `free` releases an object the
/// program must not touch again (any later access is undefined behaviour).
const NOOP_EXTERNS: &[&str] = &[
    "free",
    "srand",
    "kfree",
    "kzfree",
    "vfree",
    "kmem_cache_free",
    "devm_kfree",
];
// NOT `sprintf`: it writes its destination buffer (an unknown extern havocs it).
const PURE_EXTERNS: &[&str] = &["printf", "puts", "putchar", "fflush", "fprintf"];
/// Externs whose integer result the counterexample search pins and the replay
/// driver hands back through `--wrap` (plan 222 §2n): the string conversions,
/// character reads and socket calls that Juliet's input variants (`fgets`,
/// `listen_socket`, `connect_socket`) feed their data through. Outside the
/// counterexample search they are unknown externs like any other. Each entry
/// is `(name, C return type, C parameter list, argument list)` — what the
/// driver's `__wrap_<name>` needs; parameter types are ABI-equivalent spellings
/// that need no system header.
pub const PINNED_EXTERNS: &[(&str, &str, &str, &str)] = &[
    ("atoi", "int", "const char *a", "a"),
    ("atol", "long", "const char *a", "a"),
    ("atoll", "long long", "const char *a", "a"),
    (
        "strtol",
        "long",
        "const char *a, char **b, int c",
        "a, b, c",
    ),
    (
        "strtoll",
        "long long",
        "const char *a, char **b, int c",
        "a, b, c",
    ),
    (
        "strtoul",
        "unsigned long",
        "const char *a, char **b, int c",
        "a, b, c",
    ),
    (
        "strtoull",
        "unsigned long long",
        "const char *a, char **b, int c",
        "a, b, c",
    ),
    ("socket", "int", "int a, int b, int c", "a, b, c"),
    (
        "connect",
        "int",
        "int a, const void *b, unsigned int c",
        "a, b, c",
    ),
    (
        "bind",
        "int",
        "int a, const void *b, unsigned int c",
        "a, b, c",
    ),
    ("listen", "int", "int a, int b", "a, b"),
    (
        "accept",
        "int",
        "int a, void *b, unsigned int *c",
        "a, b, c",
    ),
    (
        "recv",
        "long",
        "int a, void *b, unsigned long c, int d",
        "a, b, c, d",
    ),
    (
        "recvfrom",
        "long",
        "int a, void *b, unsigned long c, int d, void *e, unsigned int *f",
        "a, b, c, d, e, f",
    ),
    (
        "send",
        "long",
        "int a, const void *b, unsigned long c, int d",
        "a, b, c, d",
    ),
    ("getchar", "int", "void", ""),
    ("fgetc", "int", "void *a", "a"),
    ("getc", "int", "void *a", "a"),
];

/// Whether `name` is one of the [`PINNED_EXTERNS`].
#[must_use]
pub fn is_pinned_extern(name: &str) -> bool {
    PINNED_EXTERNS.iter().any(|(n, ..)| *n == name)
}

/// Pinned externs whose result is a descriptor or a byte count: the
/// counterexample search keeps the model's value in `0..=8` (the search is a
/// restriction, never a proof), so the replay neither indexes a buffer with a
/// huge `recv` count nor closes the harness's own descriptors.
const SMALL_RESULT_EXTERNS: &[&str] = &[
    "socket", "connect", "bind", "listen", "accept", "recv", "recvfrom", "send",
];
/// Assumption intrinsics: the argument becomes a path constraint.
const ASSUME: &[&str] = &["__VERIFIER_assume", "__CPROVER_assume"];
/// libc I/O functions and the index of their `FILE *` argument.
const STREAM_ARG: &[(&str, usize)] = &[
    ("fscanf", 0),
    ("__isoc99_fscanf", 0),
    ("fprintf", 0),
    ("vfprintf", 0),
    ("fgetc", 0),
    ("getc", 0),
    ("fgets", 2),
    ("fputs", 1),
    ("fputc", 1),
    ("putc", 1),
    ("fread", 3),
    ("fwrite", 3),
    ("fflush", 0),
    ("fclose", 0),
    ("feof", 0),
    ("ferror", 0),
    ("clearerr", 0),
    ("fseek", 0),
    ("ftell", 0),
    ("rewind", 0),
    ("fileno", 0),
];
/// Externs whose control flow the havoc model cannot describe: `longjmp` does
/// not return to its caller but to the matching `setjmp`, with the globals as
/// they are THEN — a path the "returns any value" model of `setjmp` does not
/// explore. Always abstain.
const NON_LOCAL_CONTROL: &[&str] = &[
    "setjmp",
    "_setjmp",
    "sigsetjmp",
    "__sigsetjmp",
    "longjmp",
    "_longjmp",
    "siglongjmp",
];

/// Outcome of the exhaustive proof attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExhaustiveProof {
    /// Every path from `main` terminates without a feasible error call.
    Proven,
    /// Could not prove — the reason is diagnostic only (never a FALSE claim).
    Abstain(String),
}

/// A variable of a loop-header state (see [`HeaderStates`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum StateVar {
    /// An SSA value of the header's function (a header phi or a parameter).
    Value(ValueId),
    /// A scalar global (the whole object, path `[]`).
    Global(ValueId),
}

/// The EXACT set of states a proof saw at one loop header: every concrete
/// assignment to the integer variables of the footprint, as the header fixpoint
/// enumerated them. Only produced by a proof of the lazy pass on a header the
/// fixpoint stayed on, so the set over-approximates nothing and misses nothing:
/// the disjunction of these states is a loop invariant a validator can check.
/// Each state is a list of `(variable, value as bits, width)`.
/// One concrete header state: `(variable, value as bits, width)` per slot.
pub type HeaderState = Vec<(StateVar, u64, u32)>;

/// A concrete path to a violation found by [`find_counterexample`]: the values
/// every `__VERIFIER_nondet_*` (and `rand`) call returned along it, in program
/// order (`(function name, value, call instruction)`), the violating
/// instruction, and the blocks of the violating function the path entered. It
/// is a CANDIDATE: the verdict comes from replaying it natively.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Counterexample {
    pub nondets: Vec<(String, i64, InstId)>,
    pub error_inst: InstId,
    pub block_path: Vec<BlockId>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HeaderStates {
    pub func: FunctionId,
    pub block: BlockId,
    pub states: Vec<HeaderState>,
}

/// The fresh symbolic variables an encoding depends on. Feasibility queries send
/// Z3 only the path constraints connected (transitively) to the query's
/// variables: disjoint-variable constraint sets are jointly satisfiable, so a
/// component's (un)satisfiability decides the whole set's — sound AND complete.
type Vars = Rc<VarSet>;

/// Above this many symbols a dependency set becomes [`VarSet::All`]: a set
/// that large is connected to practically everything anyway, and copying it
/// at every merge and operation was measured to dominate the run time (ECA).
const VARS_CAP: usize = 64;

/// The symbols a value or constraint depends on. `All` over-approximates
/// (every constraint is then pulled into a query, which only adds precision
/// and cost, never unsoundness).
#[derive(Clone, Debug, PartialEq, Eq)]
enum VarSet {
    Few(BTreeSet<u64>),
    All,
}

impl VarSet {
    fn is_empty(&self) -> bool {
        matches!(self, VarSet::Few(set) if set.is_empty())
    }

    fn is_disjoint(&self, other: &VarSet) -> bool {
        match (self, other) {
            (VarSet::Few(a), VarSet::Few(b)) => a.is_disjoint(b),
            (VarSet::All, o) | (o, VarSet::All) => o.is_empty(),
        }
    }

    fn len(&self) -> usize {
        match self {
            VarSet::Few(set) => set.len(),
            VarSet::All => usize::MAX,
        }
    }
}

/// The empty variable set (a concrete value).
fn no_vars() -> Vars {
    Rc::new(VarSet::Few(BTreeSet::new()))
}

fn one_var(n: u64) -> Vars {
    Rc::new(VarSet::Few(BTreeSet::from([n])))
}

/// `a ∪ b`, sharing an operand when the other is empty or contained.
fn union(lhs: &Vars, rhs: &Vars) -> Vars {
    if rhs.is_empty() || Rc::ptr_eq(lhs, rhs) {
        return lhs.clone();
    }
    if lhs.is_empty() {
        return rhs.clone();
    }
    match (&**lhs, &**rhs) {
        (VarSet::Few(left), VarSet::Few(right)) => {
            if left.len() >= right.len() && right.is_subset(left) {
                return lhs.clone();
            }
            if right.len() > left.len() && left.is_subset(right) {
                return rhs.clone();
            }
            let mut u: BTreeSet<u64> = left.clone();
            u.extend(right.iter().copied());
            if u.len() > VARS_CAP && !debug_flag("nocap") {
                Rc::new(VarSet::All)
            } else {
                Rc::new(VarSet::Few(u))
            }
        }
        _ => Rc::new(VarSet::All),
    }
}

/// A modelled memory cell: a base object (an alloca or a global's address) and
/// the constant field / index path below it, as the frontend's GEP steps spell
/// it (`[]` is the object itself; `[2, 1]` is field 1 of field 2). Distinct
/// paths are distinct addresses (a union is lowered by LLVM to its largest
/// member plus pointer casts, so its members share ONE path and differ in load
/// width, which the width check catches).
type Cell = (ValueId, Vec<u32>);

/// Path step of a pointer that is somewhere INSIDE its base object but at an
/// unknown offset (a dynamic array index): loads through it are fresh, a store
/// through it havocs the whole object, and it compares equal to nothing.
const IMPRECISE: u32 = u32::MAX;

fn is_imprecise(cell: &Cell) -> bool {
    cell.1.contains(&IMPRECISE)
}

/// The cell `off` bytes into `base`. Offset 0 is the object's own cell (the
/// empty path), so the base pointer and `&obj[0]` / `&obj.first` coincide.
fn cell_at(base: ValueId, off: u32) -> Cell {
    if off == 0 {
        (base, Vec::new())
    } else {
        (base, vec![off])
    }
}

/// The byte offset of an exact cell; `None` for an unknown offset.
fn cell_offset(cell: &Cell) -> Option<u32> {
    match cell.1.as_slice() {
        [] => Some(0),
        [o] if *o != IMPRECISE => Some(*o),
        _ => None,
    }
}

/// Parse a `GEP_BYTES_KEY` extension: `(is_field, bytes)` per step.
fn parse_gep_bytes(v: &serde_json::Value) -> Option<Vec<(bool, u64)>> {
    v.as_array()?
        .iter()
        .map(|step| {
            let pair = step.as_array()?;
            let kind = pair.first()?.as_u64()?;
            let n = pair.get(1)?.as_u64()?;
            Some((kind == 1, n))
        })
        .collect()
}

/// Base of the null pointer: a KNOWN pointer that equals only itself (so
/// `p->next != NULL` is decidable once `NULL` was stored), points at no object
/// (a store through it abstains, a load is fresh), and is never allocated.
const NULL_BASE: ValueId = ValueId::new(u128::MAX);

/// A symbolic value: an exact-width bitvector (with the variables it depends
/// on), or a pointer that is either the address of a modelled cell
/// (`Some(cell)`) or opaque (`None`: a dynamic index, heap memory, an unknown
/// source). A store through an opaque pointer abstains, so opaque pointers can
/// never change a modelled cell behind the engine's back.
#[derive(Clone)]
enum Val {
    Bv(BV, Vars),
    Ptr(Option<Cell>),
}

#[derive(Clone)]
struct Frame {
    func: FunctionId,
    /// Where to resume in the caller: block, instruction index of the call.
    ret_block: BlockId,
    ret_idx: usize,
    ret_prev: Option<BlockId>,
    ret_dst: Option<ValueId>,
    /// For a recursive call: the callee's SSA bindings as they were in the
    /// outer activation, restored on return (SSA values are keyed by `ValueId`,
    /// so the inner activation overwrites them while it runs).
    saved: Vec<(ValueId, Option<Val>)>,
}

// NOTE: the four flags are independent bookkeeping bits of one path, not a
// state machine (`has_unknown`, `parked`, `parked_here`, `restarted`).
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone)]
struct State {
    func: FunctionId,
    block: BlockId,
    idx: usize,
    prev: Option<BlockId>,
    frames: Vec<Frame>,
    values: BTreeMap<ValueId, Val>,
    /// Modelled scalar cells: base (alloca / global address) → content.
    memory: BTreeMap<Cell, Val>,
    /// Cells whose address escaped (a `Gep` was taken): loads are fresh.
    escaped: BTreeSet<ValueId>,
    /// Heap objects allocated zeroed (`calloc`, `kzalloc`, …): a never-written
    /// cell reads 0. Merging keeps the intersection (an object zeroed on one
    /// side only reads fresh afterwards — any value, sound).
    zeroed: BTreeSet<ValueId>,
    constraints: Vec<(Bool, Vars)>,
    visits: BTreeMap<BlockId, u32>,
    /// A Z3 `UNKNOWN` was treated as feasible somewhere on this path, so a
    /// later feasible error call is reported as such (never as a proof).
    has_unknown: bool,
    /// `constraints.len()` when the whole path condition was last checked
    /// (lazy modes); `usize::MAX` after a merge, forcing a re-check.
    checked_len: usize,
    /// Conjunction (and variables) of the first `pc_cache.0` constraints, so
    /// the path condition is extended incrementally instead of rebuilt from
    /// thousands of constraints at every merge (hardware models: 12 s of a
    /// 60 s budget went into that rebuild).
    pc_cache: Option<(usize, Bool, Vars)>,
    /// Parked at a join point (phis done) so that sibling paths reaching the
    /// same point can be merged before execution continues.
    parked: bool,
    /// Already parked on the current block entry (park once per entry).
    parked_here: bool,
    /// Rebuilt from one concrete loop-header state by the header fixpoint
    /// (already enumerated: not enumerated again until the next block entry).
    restarted: bool,
    /// Counterexample mode only: the nondet reads of this path
    /// `(name, width, symbol index, call instruction)` and the blocks entered.
    nondets: Vec<(String, u32, u64, InstId)>,
    path: Vec<(FunctionId, BlockId)>,
    /// Symbols a branch pinned to a value on this path (they leave every term,
    /// so a model would not show them).
    pinned: BTreeMap<u64, u64>,
    /// [`Goal::MemSafety`] bookkeeping: heap blocks, dead stack objects, the
    /// live stack objects with the frame depth that owns them, the allocas
    /// executed on this path (a re-execution gets a fresh object) and the
    /// fresh objects' allocas; `dead_end` marks a path cut by a false
    /// assumption (not an execution: no leak check).
    heap: BTreeMap<ValueId, HeapObj>,
    dead: BTreeSet<ValueId>,
    stack: Vec<(usize, ValueId)>,
    seen_alloca: BTreeSet<ValueId>,
    versioned: BTreeMap<ValueId, ValueId>,
    dead_end: bool,
}

/// Concrete content of one footprint slot of a loop-header state.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum CVal {
    Bv(u64, u32),
    Ptr(Option<Cell>),
    Absent,
}

/// One call-stack frame of a concrete loop-header state: callee, return
/// point (block, instruction, previous block, result value), saved value ids.
type FrameKey = (
    FunctionId,
    BlockId,
    usize,
    Option<BlockId>,
    Option<ValueId>,
    Vec<ValueId>,
);

/// A concrete loop-header state: everything the continuation can read. The
/// structure (program point, call stack, which values / cells exist) and the
/// contents (`vals`: frames' saved values, then `values`, then `memory`, in
/// that order) are kept apart so one enumeration patches only the contents.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ConcreteKey {
    func: FunctionId,
    block: BlockId,
    idx: usize,
    frames: Vec<FrameKey>,
    value_ids: Vec<ValueId>,
    cells: Vec<Cell>,
    escaped: Vec<ValueId>,
    dead: Vec<ValueId>,
    stack: Vec<(usize, ValueId)>,
    vals: Vec<CVal>,
}

/// Abstain with a reason (used as `return Err(..)` inside the executor).
type Res<T> = Result<T, String>;

/// What counts as a violation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Goal {
    /// `unreach-call`: a feasible call of `reach_error` (or an alias).
    ReachError,
    /// `no-overflow`: a feasible signed overflow — an `nsw` add/sub/mul whose
    /// signed result wraps, `INT_MIN / -1` (`sdiv`/`srem`), or a left shift
    /// that loses bits, shifts a negative value, or shifts by ≥ width (the
    /// C standard's undefined cases; conservative where the IR has lost the
    /// signedness). A `reach_error` call is just an abort here.
    SignedOverflow,
    /// `valid-memsafety` (plan 222 §2s): every load, store, `memcpy`/`memset`
    /// lands inside a live object at a known offset; `free` gets NULL or the
    /// start of a live heap block; at every path end each unfreed heap block is
    /// reachable from the globals. Anything the engine cannot decide abstains.
    /// A `reach_error` call is an abort here.
    MemSafety,
}

/// Status of a heap block under [`Goal::MemSafety`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HeapStatus {
    Live,
    Freed,
    /// Live on one merged side, freed (or never allocated) on the other.
    Maybe,
}

/// A heap block under [`Goal::MemSafety`]: its byte size (`None` when the
/// merged sides disagree) and status.
#[derive(Clone)]
struct HeapObj {
    size: Option<(BV, Vars)>,
    status: HeapStatus,
}

/// How the executor treats the single natural loop of `main` (plan 223).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    /// Explore everything; a loop is unwound until the visit cap.
    Exhaustive,
    /// Base case of k-induction: iterations `> k` are not explored (the step
    /// case covers them); an error anywhere else abstains.
    Base { k: u32 },
    /// Step case: started from an ARBITRARY header state at iteration 1. An
    /// error, or a loop exit, in iterations `1..=k` ends the path (inductive
    /// hypothesis / irrelevant path); iteration `k + 1` and everything after
    /// its exits must be error-free; entering iteration `k + 2` ends the path.
    Step { k: u32 },
}

/// The single natural loop that k-induction reasons about: in `main`, or in a
/// loop-free `main`'s single looping callee (plan 223 slice 2).
struct LoopInfo {
    /// The function containing the loop.
    func: FunctionId,
    header: BlockId,
    /// Blocks of the loop (header included).
    body: BTreeSet<BlockId>,
    /// Index of the header's first non-phi instruction.
    first_non_phi: usize,
    /// For a callee loop: the call site in `main` (block, instruction index,
    /// result value) the step case returns to; `None` for a loop in `main`.
    call_site: Option<(BlockId, usize, Option<ValueId>)>,
}

/// Developer switch (`SAF_UNREACH_DEBUG=flag1,flag2`), for bisecting.
fn debug_flag(flag: &str) -> bool {
    static FLAGS: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    FLAGS
        .get_or_init(|| std::env::var("SAF_UNREACH_DEBUG").unwrap_or_default())
        .split(',')
        .any(|f| f == flag)
}

/// Developer trace (`SAF_UNREACH_TRACE=1`), to stderr. The message is built
/// only when tracing is on: formatting a path condition is far from free.
fn trace(msg: impl FnOnce() -> String) {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    if *ON.get_or_init(|| std::env::var_os("SAF_UNREACH_TRACE").is_some()) {
        eprintln!("[unreach_prove] {}", msg());
    }
}

/// CFG successors of a block.
fn successors(b: &saf_core::air::AirBlock) -> Vec<BlockId> {
    match b.terminator().map(|t| &t.op) {
        Some(Operation::Br { target }) => vec![*target],
        Some(Operation::CondBr {
            then_target,
            else_target,
        }) => vec![*then_target, *else_target],
        Some(Operation::Switch { default, cases }) => {
            let mut v: Vec<BlockId> = cases.iter().map(|(_, b)| *b).collect();
            v.push(*default);
            v
        }
        _ => vec![],
    }
}

/// Reverse-postorder index of every block of `f` (unreachable blocks last).
fn reverse_postorder(f: &saf_core::air::AirFunction) -> BTreeMap<BlockId, usize> {
    let blocks: BTreeMap<BlockId, &saf_core::air::AirBlock> =
        f.blocks.iter().map(|b| (b.id, b)).collect();
    let mut post: Vec<BlockId> = Vec::new();
    let mut seen: BTreeSet<BlockId> = BTreeSet::new();
    if let Some(entry) = f.entry_block.or_else(|| f.blocks.first().map(|b| b.id)) {
        let mut stack: Vec<(BlockId, Vec<BlockId>, usize)> = Vec::new();
        if let Some(eb) = blocks.get(&entry) {
            seen.insert(entry);
            stack.push((entry, successors(eb), 0));
        }
        while let Some((b, ss, i)) = stack.last_mut() {
            if *i < ss.len() {
                let t = ss[*i];
                *i += 1;
                if seen.insert(t) {
                    if let Some(tb) = blocks.get(&t) {
                        let ns = successors(tb);
                        stack.push((t, ns, 0));
                    }
                }
            } else {
                post.push(*b);
                stack.pop();
            }
        }
    }
    let mut rpo: BTreeMap<BlockId, usize> = BTreeMap::new();
    for (i, b) in post.iter().rev().enumerate() {
        rpo.insert(*b, i);
    }
    for b in &f.blocks {
        rpo.entry(b.id).or_insert(usize::MAX - 1);
    }
    rpo
}

/// The scheduler's queue: states in program order (see `Engine::order_key`),
/// so that every path reaching a join point is there before the join is
/// expanded and merging is complete, not a matter of arrival order.
struct Worklist {
    items: BTreeMap<(Vec<usize>, u64), State>,
    seq: u64,
    /// Depth-first: pop the most recently pushed state (the counterexample
    /// search, which wants ONE deep path to a violation, not a frontier).
    lifo: bool,
}

impl Worklist {
    fn push(&mut self, key: Vec<usize>, st: State) {
        self.seq += 1;
        self.items.insert((key, self.seq), st);
    }

    fn pop(&mut self) -> Option<(Vec<usize>, State)> {
        if self.lifo {
            return self.items.pop_last().map(|((k, _), st)| (k, st));
        }
        self.items.pop_first().map(|((k, _), st)| (k, st))
    }

    /// Remove and return every parked state with the same order key and the
    /// same merge key.
    fn take_mergeable(&mut self, key: &[usize], mk: &MergeKey) -> Vec<State> {
        let lo = (key.to_vec(), 0u64);
        let hi = (key.to_vec(), u64::MAX);
        let found: Vec<(Vec<usize>, u64)> = self
            .items
            .range(lo..=hi)
            .filter(|(_, st)| st.parked && !st.constraints.is_empty() && merge_key(st) == *mk)
            .map(|(k, _)| k.clone())
            .collect();
        found
            .into_iter()
            .filter_map(|k| self.items.remove(&k))
            .collect()
    }
}

/// Natural-loop headers and bodies of one function (DFS back edges).
fn natural_loops(f: &saf_core::air::AirFunction) -> Vec<(BlockId, BTreeSet<BlockId>)> {
    let succs = successors;
    let blocks: BTreeMap<BlockId, &saf_core::air::AirBlock> =
        f.blocks.iter().map(|b| (b.id, b)).collect();
    let Some(entry) = f.entry_block.or_else(|| f.blocks.first().map(|b| b.id)) else {
        return vec![];
    };
    let Some(eb) = blocks.get(&entry) else {
        return vec![];
    };
    let mut back: Vec<(BlockId, BlockId)> = Vec::new();
    let mut color: BTreeMap<BlockId, u8> = BTreeMap::new();
    let mut stack: Vec<(BlockId, Vec<BlockId>, usize)> = vec![(entry, succs(eb), 0)];
    color.insert(entry, 1);
    while let Some((b, ss, i)) = stack.last_mut() {
        if *i < ss.len() {
            let t = ss[*i];
            *i += 1;
            match color.get(&t) {
                Some(1) => back.push((*b, t)),
                Some(_) => {}
                None => {
                    if let Some(tb) = blocks.get(&t) {
                        color.insert(t, 1);
                        let ns = succs(tb);
                        stack.push((t, ns, 0));
                    }
                }
            }
        } else {
            color.insert(*b, 2);
            stack.pop();
        }
    }
    let mut preds: BTreeMap<BlockId, Vec<BlockId>> = BTreeMap::new();
    for b in &f.blocks {
        for t in succs(b) {
            preds.entry(t).or_default().push(b.id);
        }
    }
    let headers: BTreeSet<BlockId> = back.iter().map(|(_, h)| *h).collect();
    headers
        .into_iter()
        .map(|header| {
            let mut body: BTreeSet<BlockId> = BTreeSet::from([header]);
            let mut work: Vec<BlockId> = back
                .iter()
                .filter(|(_, h)| *h == header)
                .map(|(l, _)| *l)
                .collect();
            while let Some(b) = work.pop() {
                if b != header && body.insert(b) {
                    if let Some(ps) = preds.get(&b) {
                        work.extend(ps.iter().copied());
                    }
                }
            }
            (header, body)
        })
        .collect()
}

// NOTE: the flags are independent mode switches of one executor (goal pass,
// counterexample search, its order and shift policy); a mode enum would cross
// them all. Allowed as such.
#[allow(clippy::struct_excessive_bools)]
struct Engine<'a> {
    /// Solver limits for branch feasibility and for violation checks.
    params: z3::Params,
    params_violation: z3::Params,
    /// Set while a violation check runs (the longer limits apply).
    violation_query: bool,
    goal: Goal,
    mode: Mode,
    kloop: Option<LoopInfo>,
    module: &'a AirModule,
    /// Integer width of every SSA value with a known integer type.
    width: BTreeMap<ValueId, u32>,
    /// SSA values of pointer type.
    is_ptr: BTreeSet<ValueId>,
    /// Scalar global cells: address value → (initial content, is_constant).
    globals: BTreeMap<ValueId, (Option<Constant>, bool)>,
    /// Byte layouts of the modelled globals (initializer lookup by offset).
    layouts: BTreeMap<ValueId, Layout>,
    /// Exact byte sizes of the allocas the frontend could size.
    alloca_sizes: BTreeMap<ValueId, u64>,
    /// Fresh objects handed out to re-executed allocas ([`Goal::MemSafety`]).
    alloca_versions: u64,
    /// Per-step byte arithmetic of every GEP the frontend could size.
    gep_bytes: BTreeMap<InstId, Vec<(bool, u64)>>,
    /// Globals that must be treated as opaque pointers (see `Engine::new`).
    opaque_globals: BTreeSet<ValueId>,
    /// Blocks with more than one predecessor: states are merged there.
    join_blocks: BTreeSet<BlockId>,
    /// Natural-loop headers of every defined function: the lazy exhaustive
    /// pass decides a path's condition once per loop iteration there.
    loop_headers: BTreeSet<(FunctionId, BlockId)>,
    /// Exhaustive pass branching lazily (BMC-style: no per-branch solver
    /// query; one path-condition query per loop iteration and at error calls).
    lazy_exhaustive: bool,
    /// Header fixpoint (exhaustive pass): concrete loop-header states already
    /// explored, and the headers whose state proved too large to enumerate.
    fixpoint: bool,
    /// Counterexample mode: eager, no parking or merging, every path keeps its
    /// nondet reads and block trail; the first feasible violation is recorded.
    cex_mode: bool,
    cex: Option<Counterexample>,
    /// Counterexample mode: violations at these instructions are passed over
    /// (a candidate there did not replay), so the search goes on to the next
    /// feasible one on the path.
    cex_skip: BTreeSet<InstId>,
    /// Counterexample mode: depth-first worklist order.
    cex_depth_first: bool,
    /// Counterexample mode: also report `shl` overflows (the replay then runs
    /// with `-fsanitize=shift-base`); off by default because an unsigned shift
    /// looks the same in the IR and would stop the search before the real
    /// arithmetic overflow.
    cex_check_shl: bool,
    /// `(line, column)` of the program's signed left shifts, when known (see
    /// `CexOptions::signed_shl_locs`); a `shl` located elsewhere is unsigned.
    signed_shl_locs: Option<BTreeSet<(u32, u32)>>,
    /// Every fresh symbol by index (for domain splits).
    symbols: BTreeMap<u64, BV>,
    /// Symbols whose domain was found too large to split (not retried).
    no_split: BTreeSet<u64>,
    seen_header: BTreeSet<ConcreteKey>,
    fixpoint_off: BTreeSet<(FunctionId, BlockId)>,
    n_restarts: u64,
    /// SSA values each function defines (params + instruction results).
    owned: BTreeMap<FunctionId, Vec<ValueId>>,
    /// Functions containing an `alloca` (their recursion is not modelled: an
    /// inner activation's cell would alias the outer's).
    has_alloca: BTreeSet<FunctionId>,
    /// SSA values defined by an ordinary instruction and used only inside their
    /// defining block: dropped from a state when it leaves the block, so a
    /// state (and every clone and merge of it) carries the live values only —
    /// an ECA body otherwise accumulates thousands of dead temporaries.
    block_local: BTreeMap<BlockId, Vec<ValueId>>,
    /// Reverse-postorder index of every block, per function (scheduling).
    rpo: BTreeMap<(FunctionId, BlockId), usize>,
    fresh_counter: u64,
    steps: u64,
    solver_calls: u64,
    /// States popped from the worklist by the current `run`.
    states: u64,
    /// Time spent in Z3 `simplify`, `check` and in state merging (probe stats).
    t_simplify: Duration,
    t_check: Duration,
    t_merge: Duration,
    t_m_pc: Duration,
    n_merges: u64,
    deadline: Instant,
    /// One solver, reset per query (creating a solver per query dominated the
    /// cost of an unwound nondet loop).
    solver: z3::Solver,
}

#[derive(PartialEq, Eq)]
enum Feas {
    Sat,
    Unsat,
    Unknown,
}

/// Prove that no `reach_error` call is reachable from `main`, by exhaustive
/// bounded symbolic execution with exact semantics. `budget` bounds wall-clock
/// time; on expiry the result is an abstain.
#[must_use]
pub fn prove_unreach_exhaustive(module: &AirModule, budget: Duration) -> ExhaustiveProof {
    prove_exhaustive(module, budget, Goal::ReachError).0
}

/// [`prove_unreach_exhaustive`] plus the loop-header state sets the proof rests
/// on (empty unless the lazy pass with the header fixpoint produced the proof).
pub fn prove_unreach_exhaustive_with_states(
    module: &AirModule,
    budget: Duration,
) -> (ExhaustiveProof, Vec<HeaderStates>) {
    prove_exhaustive(module, budget, Goal::ReachError)
}

/// [`prove_no_overflow_exhaustive`] plus the loop-header state sets.
pub fn prove_no_overflow_exhaustive_with_states(
    module: &AirModule,
    budget: Duration,
) -> (ExhaustiveProof, Vec<HeaderStates>) {
    prove_exhaustive_opts(module, budget, Goal::SignedOverflow, None)
}

/// [`prove_no_overflow_exhaustive_with_states`] told which left shifts are
/// SIGNED (`(line, column)` locations from clang's `-fsanitize=shift-base`
/// instrumentation, see `CexOptions::signed_shl_locs`): a `shl` elsewhere is an
/// unsigned shift, defined for every operand, and is not checked. `None` checks
/// every `shl` (the IR alone cannot tell them apart).
pub fn prove_no_overflow_exhaustive_with_states_shl(
    module: &AirModule,
    budget: Duration,
    signed_shl_locs: Option<&BTreeSet<(u32, u32)>>,
) -> (ExhaustiveProof, Vec<HeaderStates>) {
    prove_exhaustive_opts(module, budget, Goal::SignedOverflow, signed_shl_locs)
}

/// [`prove_no_overflow_exhaustive`] with the signed-shift locations.
pub fn prove_no_overflow_exhaustive_shl(
    module: &AirModule,
    budget: Duration,
    signed_shl_locs: Option<&BTreeSet<(u32, u32)>>,
) -> ExhaustiveProof {
    prove_exhaustive_opts(module, budget, Goal::SignedOverflow, signed_shl_locs).0
}

/// The same engine for `valid-memsafety` (plan 222 §2s): a proof means every
/// access on every path stayed inside a live object, every `free` was valid and
/// no heap block was left unreachable from the globals at any path end (see
/// [`Goal::MemSafety`]). `malloc`/`calloc` never fail (SV-COMP rules); every
/// other allocator, and a declared `ldv_malloc` (which may fail), abstains.
#[must_use]
pub fn prove_memsafety_exhaustive(module: &AirModule, budget: Duration) -> ExhaustiveProof {
    if module
        .functions
        .iter()
        .any(|f| f.is_declaration && MAY_FAIL_MALLOC.contains(&f.name.as_str()))
    {
        return ExhaustiveProof::Abstain("ms-may-fail-allocator".to_string());
    }
    // Block-scoped objects die before their function returns; without the
    // lifetime markers of the memory-safety recipe the IR does not say when.
    let has_alloca = module.functions.iter().any(|f| {
        f.blocks.iter().any(|b| {
            b.instructions
                .iter()
                .any(|i| matches!(i.op, Operation::Alloca { .. }))
        })
    });
    if has_alloca && !module.functions.iter().any(|f| f.name == LIFETIME_START) {
        return ExhaustiveProof::Abstain("ms-no-lifetime-markers".to_string());
    }
    prove_exhaustive_opts(module, budget, Goal::MemSafety, None).0
}

/// The memory-safety ingestion recipe's names for `llvm.lifetime.start/end`
/// (the frontend drops the intrinsics; the recipe renames them to these
/// declared functions so the engine sees where each C object's lifetime
/// begins and ends).
pub const LIFETIME_START: &str = "__saf_lifetime_start";
/// See [`LIFETIME_START`].
pub const LIFETIME_END: &str = "__saf_lifetime_end";
/// The memory-safety ingestion recipe's pin: a call taking the address of a
/// local whose address escapes, so `mem2reg` leaves it a stack object. Inert.
pub const KEEP_ALIVE: &str = "__saf_keep";

/// Externs the frontend maps to a `malloc`-kind allocation whose model may
/// return NULL (the LDV harness allocators): the never-fails rule is
/// `malloc`'s, not theirs.
const MAY_FAIL_MALLOC: &[&str] = &["ldv_malloc"];

/// Does a `printf` format contain a `%n` conversion (which writes through
/// its argument)? Conservative: any `%` followed by flags, width, precision
/// or length characters and then `n`.
fn format_writes(fmt: &str) -> bool {
    let b = fmt.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' {
            let mut j = i + 1;
            while j < b.len() && b"-+ #0123456789.*hlLqjzt'".contains(&b[j]) {
                j += 1;
            }
            if j < b.len() && b[j] == b'n' {
                return true;
            }
            // `%%` and every other conversion: continue after it.
            i = j + 1;
            continue;
        }
        i += 1;
    }
    false
}

/// Does any function of the module allocate on the heap?
fn module_allocates_heap(module: &AirModule) -> bool {
    module.functions.iter().any(|f| {
        f.blocks.iter().any(|b| {
            b.instructions
                .iter()
                .any(|i| matches!(i.op, Operation::HeapAlloc { .. }))
        })
    })
}

/// The same engine for `no-overflow`: a proof means every path terminated
/// without a feasible signed overflow (see [`Goal::SignedOverflow`] for what
/// is checked; `reach_error` calls merely abort). The frontend keeps LLVM's
/// `nsw` flag on signed `add`/`sub`/`mul` (`Instruction::has_no_signed_wrap`),
/// which is how signed arithmetic is told apart from unsigned in the IR.
/// Everything said about soundness, gates and budgets for
/// [`prove_unreach_exhaustive`] applies unchanged.
pub fn prove_no_overflow_exhaustive(module: &AirModule, budget: Duration) -> ExhaustiveProof {
    prove_exhaustive_opts(module, budget, Goal::SignedOverflow, None).0
}

/// A concrete violation path for `unreach-call` (`overflow == false`) or
/// `no-overflow` (`true`), or `None` within `budget`: the executor runs
/// eagerly without merging so every path keeps its own nondet trail, and the
/// first feasible violation's model fixes the inputs. The result is a replay
/// CANDIDATE only — nothing here decides a verdict.
/// Knobs of the counterexample search (see [`find_counterexample_excluding`]).
#[derive(Debug, Default, Clone)]
pub struct CexOptions {
    /// Violations at these instructions are passed over (a candidate there did
    /// not replay).
    pub skip: BTreeSet<InstId>,
    /// Newest state first (a deep violation behind a wide frontier).
    pub depth_first: bool,
    /// Also report `shl` overflows (confirmed by `-fsanitize=shift-base`).
    pub check_shl: bool,
    /// Source locations `(line, column)` of the program's SIGNED left shifts
    /// (from clang's `-fsanitize=shift-base` instrumentation); a `shl` at any
    /// other location is unsigned and never an overflow. `None`: unknown, every
    /// `shl` is checked.
    pub signed_shl_locs: Option<BTreeSet<(u32, u32)>>,
}

#[must_use]
pub fn find_counterexample(
    module: &AirModule,
    budget: Duration,
    overflow: bool,
) -> Option<Counterexample> {
    find_counterexample_excluding(module, budget, overflow, &CexOptions::default())
}

/// [`find_counterexample`] that passes over violations at the `skip`
/// instructions: a caller whose replay of an earlier candidate failed (an
/// environment the driver cannot reproduce, an unsigned shift the IR cannot
/// tell from a signed one) asks for the next feasible violation on the path.
/// `depth_first` explores the newest state first — the right order for a deep
/// violation behind a wide frontier (xcsp, sequentialised threads), the wrong
/// one for a reactive loop that never returns (ECA), hence a caller's choice.
/// `check_shl` also reports left-shift overflows (last resort: the IR cannot
/// tell a signed shift from an unsigned one, so these candidates often do not
/// replay and would otherwise mask the arithmetic overflow behind them).
#[must_use]
pub fn find_counterexample_excluding(
    module: &AirModule,
    budget: Duration,
    overflow: bool,
    opts: &CexOptions,
) -> Option<Counterexample> {
    let main = module
        .functions
        .iter()
        .find(|f| f.name == "main" && !f.is_declaration)?;
    let entry = main
        .entry_block
        .or_else(|| main.blocks.first().map(|b| b.id))?;
    let mut engine = Engine::new(module, budget);
    engine.goal = if overflow {
        Goal::SignedOverflow
    } else {
        Goal::ReachError
    };
    engine.cex_mode = true;
    engine.cex_depth_first = opts.depth_first;
    engine.cex_check_shl = opts.check_shl;
    engine.cex_skip.clone_from(&opts.skip);
    engine.signed_shl_locs.clone_from(&opts.signed_shl_locs);
    engine.deadline = Instant::now() + budget;
    let init = engine.initial_state(main, entry);
    let res = engine.run(init);
    trace(|| {
        format!(
            "counterexample search ended: {res:?} (cex: {})",
            engine.cex.is_some()
        )
    });
    match res {
        Err(reason) if reason == "error-feasible" => engine.cex.take(),
        Ok(()) | Err(_) => None,
    }
}

fn prove_exhaustive(
    module: &AirModule,
    budget: Duration,
    goal: Goal,
) -> (ExhaustiveProof, Vec<HeaderStates>) {
    prove_exhaustive_opts(module, budget, goal, None)
}

fn prove_exhaustive_opts(
    module: &AirModule,
    budget: Duration,
    goal: Goal,
    signed_shl_locs: Option<&BTreeSet<(u32, u32)>>,
) -> (ExhaustiveProof, Vec<HeaderStates>) {
    let Some(main) = module
        .functions
        .iter()
        .find(|f| f.name == "main" && !f.is_declaration)
    else {
        return (ExhaustiveProof::Abstain("no-main".to_string()), Vec::new());
    };
    let Some(entry) = main
        .entry_block
        .or_else(|| main.blocks.first().map(|b| b.id))
    else {
        return (ExhaustiveProof::Abstain("no-entry".to_string()), Vec::new());
    };
    let setup = Instant::now();
    let mut engine = Engine::new(module, budget);
    engine.goal = goal;
    engine.signed_shl_locs = signed_shl_locs.cloned();
    trace(|| format!("engine ready after {:?}", setup.elapsed()));
    // The budget is exploration time: table building on a 170 k-line ECA
    // module takes seconds and is not charged.
    let start = Instant::now();
    engine.deadline = start + budget;
    // k-induction first when `main` has a single natural loop (plan 223): its
    // base and step cases are bounded (≤ K_MAX + 1 iterations each), so they are
    // cheap, and on a loop fed fresh nondet input every iteration they are the
    // only route to a proof — the exhaustive pass would only burn the budget on
    // such a loop. It gets at most half the budget; the exhaustive pass gets
    // everything that is left.
    // `valid-memsafety` on a program that allocates on the heap gets neither
    // k-induction (an arbitrary header state has no heap bookkeeping) nor the
    // loop-header fixpoint: the leak check runs at path ends, so every path
    // must really end (see `Engine::ms_memtrack`). Without a heap both are
    // sound for the access checks: the arbitrary header state binds only the
    // allocas of the functions live during the loop (live objects), and every
    // other pointer in it is opaque (an access through it abstains).
    let memsafety = goal == Goal::MemSafety;
    let heap_module = memsafety && module_allocates_heap(module);
    let loop_shape = if heap_module {
        Err(String::new())
    } else {
        Engine::find_induction_loop(module, main)
    };
    trace(|| format!("induction loop search done after {:?}", start.elapsed()));
    let induction_note = match loop_shape {
        // No loop anywhere: nothing to say about induction.
        Err(why) if why.is_empty() => String::new(),
        Err(why) => format!("; k-induction: {why}"),
        Ok(lp) => {
            engine.kloop = Some(lp);
            engine.deadline = start + budget / 2;
            let outcome = engine.k_induction(main, entry);
            engine.kloop = None;
            engine.mode = Mode::Exhaustive;
            engine.steps = 0;
            engine.solver_calls = 0;
            engine.states = 0;
            match outcome {
                Ok(_k) => return (ExhaustiveProof::Proven, Vec::new()),
                Err(e) => format!("; k-induction: {e}"),
            }
        }
    };
    // Exhaustive pass, twice: lazily (one query per loop iteration and per
    // error call; cheaper on constraint-heavy programs such as XCSP) with the
    // header fixpoint, on half of what is left; then eagerly (a query per
    // branch; cheaper where most branches are dead) without it, on the rest.
    // Measured 2026-10-10 on the 1,325 tasks proven before: 24 only by the
    // eager pass, 9 XCSP tasks only by the lazy one; the fixpoint's enumeration
    // cost 3 BitVectors proofs when it ran in both passes (plan 222 §2f).
    let left = budget.saturating_sub(start.elapsed());
    engine.fixpoint = !heap_module;
    engine.lazy_exhaustive = true;
    engine.deadline = Instant::now() + left / 2;
    let init = engine.initial_state(main, entry);
    let lazy_note = match engine.run(init) {
        Ok(()) => {
            let states = engine.header_states();
            return (ExhaustiveProof::Proven, states);
        }
        // A feasible error path is final: the eager pass would find it again.
        Err(reason) if reason == "error-feasible" => {
            return (
                ExhaustiveProof::Abstain(format!("{reason}{induction_note}")),
                Vec::new(),
            );
        }
        Err(reason) => reason,
    };
    engine.lazy_exhaustive = false;
    engine.fixpoint = false;
    engine.seen_header.clear();
    engine.fixpoint_off.clear();
    engine.steps = 0;
    engine.solver_calls = 0;
    engine.states = 0;
    engine.n_restarts = 0;
    engine.deadline = start + budget;
    let init = engine.initial_state(main, entry);
    let proof = match engine.run(init) {
        Ok(()) => ExhaustiveProof::Proven,
        Err(reason) if reason == lazy_note => {
            ExhaustiveProof::Abstain(format!("{reason}{induction_note}"))
        }
        Err(reason) => {
            ExhaustiveProof::Abstain(format!("{reason}; lazy: {lazy_note}{induction_note}"))
        }
    };
    (proof, Vec::new())
}

impl<'a> Engine<'a> {
    // NOTE: one pass over the module collecting every table the executor reads
    // (widths, pointer values, owned values, allocas, globals, join blocks); the
    // tables are built together because they come from the same traversal.
    #[allow(clippy::too_many_lines)]
    fn new(module: &'a AirModule, budget: Duration) -> Self {
        let mut width = BTreeMap::new();
        let mut is_ptr = BTreeSet::new();
        let mut owned: BTreeMap<FunctionId, Vec<ValueId>> = BTreeMap::new();
        let mut has_alloca = BTreeSet::new();
        let mut def_block: BTreeMap<ValueId, BlockId> = BTreeMap::new();
        let mut candidates: BTreeSet<ValueId> = BTreeSet::new();
        let mut cross_block: BTreeSet<ValueId> = BTreeSet::new();
        for f in &module.functions {
            for b in &f.blocks {
                for inst in &b.instructions {
                    if let Some(d) = inst.dst {
                        def_block.insert(d, b.id);
                        if !matches!(inst.op, Operation::Alloca { .. } | Operation::Phi { .. }) {
                            candidates.insert(d);
                        }
                    }
                }
            }
        }
        for f in &module.functions {
            for b in &f.blocks {
                for inst in &b.instructions {
                    if let Operation::Phi { incoming } = &inst.op {
                        cross_block.extend(incoming.iter().map(|(_, v)| *v));
                    }
                    for &o in &inst.operands {
                        if def_block.get(&o).is_some_and(|db| *db != b.id) {
                            cross_block.insert(o);
                        }
                    }
                }
            }
        }
        // Per defining block: an inlined callee leaving one of ITS blocks must
        // not drop the caller's still-live values (lost the XCSP proofs).
        let mut block_local: BTreeMap<BlockId, Vec<ValueId>> = BTreeMap::new();
        for v in candidates.difference(&cross_block) {
            if let Some(b) = def_block.get(v) {
                block_local.entry(*b).or_default().push(*v);
            }
        }
        let mut rpo: BTreeMap<(FunctionId, BlockId), usize> = BTreeMap::new();
        for f in &module.functions {
            for (b, i) in reverse_postorder(f) {
                rpo.insert((f.id, b), i);
            }
        }
        let classify = |ty: Option<saf_core::ids::TypeId>| -> (Option<u32>, bool) {
            match ty.and_then(|t| module.get_type(t)) {
                Some(AirType::Integer { bits }) => (Some(u32::from(*bits)), false),
                // A float is carried as the bitvector of its IEEE encoding; the
                // float operations go through Z3's floating-point sort (`fpa`);
                // x87 `long double` (80 bits) is repacked to Z3's (15, 64) sort.
                // 128-bit floats have no model: no width, abstain.
                Some(AirType::Float { bits }) if matches!(*bits, 32 | 64 | 80) => {
                    (Some(u32::from(*bits)), false)
                }
                Some(AirType::Pointer | AirType::Reference { .. }) => (None, true),
                _ => (None, false),
            }
        };
        for f in &module.functions {
            let own = owned.entry(f.id).or_default();
            for p in &f.params {
                own.push(p.id);
                let (w, ptr) = classify(p.param_type);
                if let Some(w) = w {
                    width.insert(p.id, w);
                }
                if ptr {
                    is_ptr.insert(p.id);
                }
            }
            for b in &f.blocks {
                for i in &b.instructions {
                    if matches!(i.op, Operation::Alloca { .. }) {
                        has_alloca.insert(f.id);
                    }
                    if let Some(d) = i.dst {
                        own.push(d);
                        let (w, ptr) = classify(i.result_type);
                        if let Some(w) = w {
                            width.insert(d, w);
                        }
                        if ptr || matches!(i.op, Operation::Alloca { .. } | Operation::Gep { .. }) {
                            is_ptr.insert(d);
                        }
                    }
                }
            }
        }
        // Integer constants carry their own width; an operand with no definition
        // (an `undef`) borrows the width of the constant it is combined with.
        for (vid, c) in &module.constants {
            if let Constant::Int { bits, .. } = c {
                width.insert(*vid, u32::from(*bits));
            }
        }
        // A global the frontend collapsed a constant pointer expression onto is
        // opaque (its base `ValueId` may stand for ANY offset); if a collapse
        // could not be attributed, every global is.
        let opaque_globals: BTreeSet<ValueId> = match &module.collapsed_globals {
            Some(set) => set.clone(),
            None => module.globals.iter().map(|g| g.id).collect(),
        };
        let globals = module
            .globals
            .iter()
            .filter(|g| !opaque_globals.contains(&g.id))
            .map(|g| (g.id, (g.init.clone(), g.is_constant)))
            .collect();
        let layouts = module
            .globals
            .iter()
            .filter_map(|g| g.layout.clone().map(|l| (g.id, l)))
            .collect();
        let mut gep_bytes = BTreeMap::new();
        let mut alloca_sizes = BTreeMap::new();
        for f in &module.functions {
            for b in &f.blocks {
                for i in &b.instructions {
                    if let Some(steps) = i.extensions.get(GEP_BYTES_KEY).and_then(parse_gep_bytes) {
                        gep_bytes.insert(i.id, steps);
                    }
                    if let (Operation::Alloca { .. }, Some(d)) = (&i.op, i.dst) {
                        if let Some(n) = i
                            .extensions
                            .get(saf_core::air::ALLOCA_LAYOUT_SIZE_KEY)
                            .or_else(|| i.extensions.get(saf_core::air::ALLOCA_EXACT_SIZE_KEY))
                            .and_then(serde_json::Value::as_u64)
                        {
                            alloca_sizes.insert(d, n);
                        }
                    }
                }
            }
        }
        let mut preds: BTreeMap<BlockId, u32> = BTreeMap::new();
        for f in &module.functions {
            for b in &f.blocks {
                if let Some(t) = b.terminator() {
                    let succs: Vec<BlockId> = match &t.op {
                        Operation::Br { target } => vec![*target],
                        Operation::CondBr {
                            then_target,
                            else_target,
                        } => vec![*then_target, *else_target],
                        Operation::Switch { default, cases } => {
                            let mut v: Vec<BlockId> = cases.iter().map(|(_, b)| *b).collect();
                            v.push(*default);
                            v
                        }
                        _ => vec![],
                    };
                    for sb in succs {
                        *preds.entry(sb).or_insert(0) += 1;
                    }
                }
            }
        }
        let join_blocks = preds
            .into_iter()
            .filter(|(_, n)| *n > 1)
            .map(|(b, _)| b)
            .collect();
        let mut loop_headers: BTreeSet<(FunctionId, BlockId)> = BTreeSet::new();
        for f in &module.functions {
            if !f.is_declaration {
                loop_headers.extend(natural_loops(f).into_iter().map(|(h, _)| (f.id, h)));
            }
        }
        let solver = z3::Solver::new();
        let mut params = z3::Params::new();
        params.set_u32("timeout", Z3_TIMEOUT_MS);
        params.set_u32("rlimit", Z3_RLIMIT);
        params.set_u32("random_seed", Z3_SEED);
        solver.set_params(&params);
        let mut params_violation = z3::Params::new();
        params_violation.set_u32("timeout", Z3_VIOLATION_TIMEOUT_MS);
        params_violation.set_u32("rlimit", Z3_RLIMIT * 6);
        params_violation.set_u32("random_seed", Z3_SEED);
        Self {
            params,
            params_violation,
            violation_query: false,
            goal: Goal::ReachError,
            mode: Mode::Exhaustive,
            kloop: None,
            module,
            width,
            is_ptr,
            globals,
            layouts,
            gep_bytes,
            alloca_sizes,
            alloca_versions: 0,
            opaque_globals,
            join_blocks,
            loop_headers,
            lazy_exhaustive: false,
            fixpoint: false,
            cex_mode: false,
            cex: None,
            cex_skip: BTreeSet::new(),
            cex_depth_first: false,
            cex_check_shl: false,
            signed_shl_locs: None,
            symbols: BTreeMap::new(),
            no_split: BTreeSet::new(),
            seen_header: BTreeSet::new(),
            fixpoint_off: BTreeSet::new(),
            n_restarts: 0,
            owned,
            has_alloca,
            block_local,
            rpo,
            fresh_counter: 0,
            steps: 0,
            solver_calls: 0,
            states: 0,
            t_simplify: Duration::ZERO,
            t_check: Duration::ZERO,
            t_merge: Duration::ZERO,
            t_m_pc: Duration::ZERO,
            n_merges: 0,
            deadline: Instant::now() + budget,
            solver,
        }
    }

    /// Record that `st` took a branch under `cond`: a condition that pins a
    /// symbol (`input == 3`) concretises it (every later test on it folds
    /// instead of costing a solver query), a condition that is literally true
    /// is not recorded at all (a concrete path keeps an EMPTY condition), and
    /// anything else joins the path constraints.
    fn take_branch(st: &mut State, cond: Bool, vars: Vars) {
        if let Some((n, sym, k)) = symbol_equality(&cond).filter(|_| !debug_flag("nopin")) {
            trace(|| format!("pin s{n} := {k}"));
            // The symbol is gone from every value, cell and constraint of the
            // path: `sym == k` itself has nothing left to constrain.
            Self::concretize(st, n, &sym, &k);
            if !debug_flag("pinkeep") {
                return;
            }
        }
        if vars.is_empty() && cond.simplify().as_bool() == Some(true) {
            return;
        }
        st.constraints.push((cond, vars));
    }

    /// Replace symbol `n` (`sym`) by the numeral `k` in every value and cell of
    /// `st` that depends on it — exact under the constraint `sym == k` the path
    /// has just taken. Values that no longer depend on any symbol fold to
    /// numerals, so later branches on them need no solver.
    fn concretize(st: &mut State, n: u64, sym: &BV, k: &BV) {
        if let Some(v) = k.as_u64() {
            st.pinned.insert(n, v);
        }
        let depends = |vs: &Vars| match &**vs {
            VarSet::Few(set) => set.contains(&n),
            VarSet::All => true,
        };
        let drop = |vs: &Vars| -> Vars {
            match &**vs {
                VarSet::Few(set) if set.contains(&n) => {
                    let mut set = set.clone();
                    set.remove(&n);
                    Rc::new(VarSet::Few(set))
                }
                _ => vs.clone(),
            }
        };
        for h in st.heap.values_mut() {
            if let Some((bv, vs)) = &mut h.size {
                if depends(vs) {
                    *bv = bv.substitute(&[(sym, k)]).simplify();
                    *vs = drop(vs);
                }
            }
        }
        for v in st.values.values_mut().chain(st.memory.values_mut()) {
            if let Val::Bv(bv, vs) = v {
                if depends(vs) {
                    *bv = bv.substitute(&[(sym, k)]).simplify();
                    *vs = drop(vs);
                }
            }
        }
        // The path constraints too: those that become literally true leave
        // (a fully concrete path then has an EMPTY condition and its header
        // and error checks need no solver); a literal false stays and kills
        // the path at the next check.
        let mut kept: Vec<(Bool, Vars)> = Vec::with_capacity(st.constraints.len());
        for (b, vs) in st.constraints.drain(..) {
            if depends(&vs) {
                let nb = b.substitute(&[(sym, k)]).simplify();
                if nb.as_bool() == Some(true) {
                    continue;
                }
                kept.push((nb, drop(&vs)));
            } else {
                kept.push((b, vs));
            }
        }
        st.constraints = kept;
        st.checked_len = st.checked_len.min(st.constraints.len());
        st.pc_cache = None;
        if debug_flag("pincheck") {
            let probe = BV::fresh_const("pincheck", sym.get_size());
            let mentions = |bv: &BV| bv.substitute(&[(sym, &probe)]) != *bv;
            for (vid, v) in &st.values {
                if let Val::Bv(bv, vs) = v {
                    if mentions(bv) {
                        trace(|| {
                            format!(
                                "PINCHECK value {vid:?} still mentions s{n} (vars {})",
                                vs.len()
                            )
                        });
                    }
                }
            }
            for (cell, v) in &st.memory {
                if let Val::Bv(bv, vs) = v {
                    if mentions(bv) {
                        trace(|| {
                            format!(
                                "PINCHECK cell {cell:?} still mentions s{n} (vars {})",
                                vs.len()
                            )
                        });
                    }
                }
            }
            for (b, vs) in &st.constraints {
                let mb = b.substitute(&[(sym, &probe)]) != *b;
                if mb {
                    trace(|| {
                        format!(
                            "PINCHECK constraint still mentions s{n} (vars {}): {b}",
                            vs.len()
                        )
                    });
                }
            }
            for f in &st.frames {
                for (vid, v) in &f.saved {
                    if let Some(Val::Bv(bv, _)) = v {
                        if mentions(bv) {
                            trace(|| format!("PINCHECK saved {vid:?} still mentions s{n}"));
                        }
                    }
                }
            }
        }
    }

    fn fresh(&mut self, bits: u32) -> (BV, Vars) {
        let n = self.fresh_counter;
        self.fresh_counter += 1;
        let sym = BV::new_const(format!("s{n}"), bits);
        self.symbols.insert(n, sym.clone());
        (sym, one_var(n))
    }

    /// Most values a symbol may take for a branch on it to be split into one
    /// concrete state per value.
    const SPLIT_DOMAIN: usize = 8;

    /// Lazy modes: a branch whose condition depends on ONE symbol whose path
    /// constraints leave it at most [`Self::SPLIT_DOMAIN`] values (ECA's
    /// `1 <= input <= 6`) is better taken concretely: the conditions returned
    /// pin the symbol to each value (all-SAT over its constraint component,
    /// exhaustive by construction), and every later test on it folds. `None`
    /// when the domain is larger (remembered) or the query fails.
    fn small_domain_split(&mut self, st: &State, vars: &Vars) -> Res<Option<Vec<(Bool, Vars)>>> {
        let VarSet::Few(set) = &**vars else {
            return Ok(None);
        };
        if set.len() != 1 {
            return Ok(None);
        }
        let n = *set.iter().next().ok_or("split-empty")?;
        if self.no_split.contains(&n) {
            return Ok(None);
        }
        let Some(sym) = self.symbols.get(&n).cloned() else {
            return Ok(None);
        };
        // The constraint component of the symbol (as `feasible` slices it).
        let mut frontier: Vars = vars.clone();
        let mut chosen: Vec<usize> = Vec::new();
        let mut taken = vec![false; st.constraints.len()];
        loop {
            let mut grew = false;
            for (i, (_, cv)) in st.constraints.iter().enumerate() {
                if !taken[i] && !cv.is_disjoint(&frontier) {
                    taken[i] = true;
                    chosen.push(i);
                    let before = frontier.len();
                    frontier = union(&frontier, cv);
                    grew |= frontier.len() > before || matches!(*frontier, VarSet::All);
                }
            }
            if !grew {
                break;
            }
        }
        self.solver.reset();
        for i in chosen {
            self.solver.assert(&st.constraints[i].0);
        }
        let mut values: Vec<u64> = Vec::new();
        loop {
            if values.len() >= Self::SPLIT_DOMAIN {
                self.no_split.insert(n);
                return Ok(None);
            }
            self.solver_calls += 1;
            let t0 = Instant::now();
            let res = self.solver.check();
            self.t_check += t0.elapsed();
            match res {
                SatResult::Unsat => break,
                SatResult::Unknown => {
                    self.no_split.insert(n);
                    return Ok(None);
                }
                SatResult::Sat => {
                    let model = self.solver.get_model().ok_or("split-no-model")?;
                    let v = model
                        .eval(&sym, true)
                        .and_then(|b| b.as_u64())
                        .ok_or("split-model-eval")?;
                    values.push(v);
                    self.solver
                        .assert(eq(&sym, &BV::from_u64(v, sym.get_size())).not());
                }
            }
        }
        trace(|| format!("domain split: s{n} ∈ {values:?}"));
        Ok(Some(
            values
                .into_iter()
                .map(|v| (eq(&sym, &BV::from_u64(v, sym.get_size())), vars.clone()))
                .collect(),
        ))
    }

    /// A fresh Boolean symbol (merge guard).
    fn fresh_bool(&mut self) -> (Bool, Vars) {
        let n = self.fresh_counter;
        self.fresh_counter += 1;
        (Bool::new_const(format!("g{n}")), one_var(n))
    }

    /// A fresh symbol wrapped as a value.
    fn fresh_val(&mut self, bits: u32) -> Val {
        let (b, vs) = self.fresh(bits);
        Val::Bv(b, vs)
    }

    fn block(&self, func: FunctionId, id: BlockId) -> Res<&'a saf_core::air::AirBlock> {
        self.module
            .function(func)
            .and_then(|f| f.blocks.iter().find(|b| b.id == id))
            .ok_or_else(|| "block-missing".to_string())
    }

    /// The real initial state: `main`'s entry with its parameters unconstrained.
    fn initial_state(&mut self, main: &saf_core::air::AirFunction, entry: BlockId) -> State {
        let mut init = State {
            func: main.id,
            block: entry,
            idx: 0,
            prev: None,
            frames: Vec::new(),
            values: BTreeMap::new(),
            memory: BTreeMap::new(),
            escaped: BTreeSet::new(),
            zeroed: BTreeSet::new(),
            constraints: Vec::new(),
            visits: BTreeMap::new(),
            has_unknown: false,
            checked_len: 0,
            pc_cache: None,
            parked: false,
            parked_here: false,
            restarted: false,
            nondets: Vec::new(),
            path: Vec::new(),
            pinned: BTreeMap::new(),
            heap: BTreeMap::new(),
            dead: BTreeSet::new(),
            stack: Vec::new(),
            seen_alloca: BTreeSet::new(),
            versioned: BTreeMap::new(),
            dead_end: false,
        };
        for p in &main.params {
            let v = match self.width.get(&p.id) {
                Some(&w) => self.fresh_val(w),
                None => Val::Ptr(None),
            };
            init.values.insert(p.id, v);
        }
        init
    }

    /// The loop k-induction reasons about (plan 223): the single natural loop of
    /// `main`; or, when `main` is loop-free, the single natural loop of the one
    /// looping function `main` calls directly, exactly once, with no other
    /// looping callee (`s3_srvr`: `main` → `ssl3_accept`). Nested or multiple
    /// loops → `None`.
    fn find_induction_loop(
        module: &AirModule,
        main: &saf_core::air::AirFunction,
    ) -> Result<LoopInfo, String> {
        let first_non_phi = |f: &saf_core::air::AirFunction, header: BlockId| -> Option<usize> {
            f.blocks
                .iter()
                .find(|b| b.id == header)?
                .instructions
                .iter()
                .position(|i| !matches!(i.op, Operation::Phi { .. }))
        };
        let own = natural_loops(main);
        if own.len() == 1 {
            let (header, body) = own.into_iter().next().ok_or("no-loop")?;
            return Ok(LoopInfo {
                func: main.id,
                header,
                first_non_phi: first_non_phi(main, header).ok_or("no-loop")?,
                body,
                call_site: None,
            });
        }
        if !own.is_empty() {
            return Err(format!("main-has-{}-loops", own.len()));
        }
        // Loop-free main: its direct callees that contain loops.
        let mut looping: Vec<(FunctionId, BlockId, usize, Option<ValueId>)> = Vec::new();
        for b in &main.blocks {
            for (idx, inst) in b.instructions.iter().enumerate() {
                if let Operation::CallDirect { callee } = inst.op {
                    let Some(f) = module.function(callee) else {
                        continue;
                    };
                    if !f.is_declaration && !natural_loops(f).is_empty() {
                        looping.push((callee, b.id, idx, inst.dst));
                    }
                }
            }
        }
        if looping.is_empty() {
            return Err(String::new()); // no loop anywhere
        }
        if looping.len() != 1 {
            return Err(format!("main-calls-{}-looping-callees", looping.len()));
        }
        let (callee, cb, cidx, cdst) = looping[0];
        let f = module.function(callee).ok_or("no-loop")?;
        let loops = natural_loops(f);
        if loops.len() != 1 {
            return Err(format!("callee-has-{}-loops", loops.len()));
        }
        let (header, body) = loops.into_iter().next().ok_or("no-loop")?;
        Ok(LoopInfo {
            func: callee,
            header,
            first_non_phi: first_non_phi(f, header).ok_or("no-loop")?,
            body,
            call_site: Some((cb, cidx, cdst)),
        })
    }

    /// k-induction over the loop in `self.kloop` (plan 223): for k = 1..=K_MAX,
    /// the base case (unwind k iterations from the real initial state) and the
    /// step case (k + 1 iterations from an arbitrary header state, the first k
    /// assumed error-free) must both be error-free. Returns the `k` that worked.
    fn k_induction(&mut self, main: &saf_core::air::AirFunction, entry: BlockId) -> Res<u32> {
        let mut last = String::from("not-k-inductive");
        for k in 1..=K_MAX {
            self.mode = Mode::Base { k };
            let init = self.initial_state(main, entry);
            let base = self.run(init);
            trace(|| format!("base k={k}: {base:?}"));
            base?; // an error within k iterations, or a budget: abstain
            self.mode = Mode::Step { k };
            let s0 = self.arbitrary_header_state(main);
            trace(|| {
                format!(
                    "step k={k}: start block={:?} idx={} escaped={} visits={:?}",
                    s0.block,
                    s0.idx,
                    s0.escaped.len(),
                    s0.visits
                )
            });
            let step = self.run(s0);
            trace(|| format!("step k={k}: {step:?}"));
            match step {
                Ok(()) => return Ok(k),
                Err(e) if e == "error-feasible" => last = e,
                // A solver time-out will not get easier with a longer unrolling.
                Err(e) if e == "error-unknown" => return Err(format!("error-unknown-at-k{k}")),
                Err(e) => return Err(e),
            }
        }
        Err(format!("{last}-up-to-k{K_MAX}"))
    }

    /// An ARBITRARY state at the loop header, iteration 1 of the step case:
    /// every header phi is a fresh symbol, every alloca of `main` is a cell whose
    /// content is unknown (escaped: a load is fresh until a store), every
    /// mutable global likewise; constant globals keep their initialiser. Values
    /// defined before the loop become fresh symbols on first use (`val`).
    fn arbitrary_header_state(&mut self, main: &saf_core::air::AirFunction) -> State {
        let (lfunc, header, first_non_phi, call_site) = {
            let lp = self.kloop.as_ref().expect("k-induction needs a loop");
            (lp.func, lp.header, lp.first_non_phi, lp.call_site)
        };
        let frames = match call_site {
            Some((cb, cidx, cdst)) => vec![Frame {
                func: main.id,
                ret_block: cb,
                ret_idx: cidx,
                ret_prev: None,
                ret_dst: cdst,
                saved: Vec::new(),
            }],
            None => Vec::new(),
        };
        let mut st = State {
            func: lfunc,
            block: header,
            idx: first_non_phi,
            prev: None,
            frames,
            values: BTreeMap::new(),
            memory: BTreeMap::new(),
            escaped: BTreeSet::new(),
            zeroed: BTreeSet::new(),
            constraints: Vec::new(),
            // Iteration 1. When the header has no phis the state starts at index
            // 0 and the entry bookkeeping below counts this entry itself.
            visits: BTreeMap::from([(header, u32::from(first_non_phi != 0))]),
            has_unknown: false,
            checked_len: 0,
            pc_cache: None,
            parked: false,
            parked_here: true,
            restarted: false,
            nondets: Vec::new(),
            path: Vec::new(),
            pinned: BTreeMap::new(),
            heap: BTreeMap::new(),
            dead: BTreeSet::new(),
            stack: Vec::new(),
            seen_alloca: BTreeSet::new(),
            versioned: BTreeMap::new(),
            dead_end: false,
        };
        // Parameters of main and of the loop's function are unconstrained.
        let mut param_ids: Vec<ValueId> = main.params.iter().map(|p| p.id).collect();
        if let Some(f) = self.module.function(lfunc) {
            param_ids.extend(f.params.iter().map(|p| p.id));
        }
        for pid in param_ids {
            let v = match self.width.get(&pid) {
                Some(&w) => self.fresh_val(w),
                None => Val::Ptr(None),
            };
            st.values.insert(pid, v);
        }
        let funcs: Vec<FunctionId> = if lfunc == main.id {
            vec![main.id]
        } else {
            vec![main.id, lfunc]
        };
        for fid in funcs {
            let Some(f) = self.module.function(fid) else {
                continue;
            };
            let mut binds: Vec<(ValueId, Val)> = Vec::new();
            for b in &f.blocks {
                for i in &b.instructions {
                    match (&i.op, i.dst) {
                        (Operation::Alloca { .. }, Some(d)) => {
                            binds.push((d, Val::Ptr(Some((d, Vec::new())))));
                            st.escaped.insert(d);
                        }
                        (Operation::Phi { .. }, Some(d)) if b.id == header => {
                            let v = match self.width.get(&d) {
                                Some(&w) => self.fresh_val(w),
                                None => Val::Ptr(None),
                            };
                            binds.push((d, v));
                        }
                        _ => {}
                    }
                }
            }
            for (d, v) in binds {
                st.values.insert(d, v);
            }
        }
        let mutable_globals: Vec<ValueId> = self
            .globals
            .iter()
            .filter(|(_, (_, is_const))| !*is_const)
            .map(|(g, _)| *g)
            .collect();
        st.escaped.extend(mutable_globals);
        st
    }

    /// Exhaustively explore from `init`. `Ok(())` ⇒ no feasible error call.
    ///
    /// Breadth-first, so sibling paths tend to be pending together when they
    /// reach a join point; a state parked there absorbs every pending state with
    /// the same key (see [`merge_key`]) before it continues. Merging is exact —
    /// the path condition becomes a disjunction and differing values become
    /// `ite`s — and turns the exponential path count of a branchy state machine
    /// into a linear-size formula.
    fn run(&mut self, init: State) -> Res<()> {
        let mut worklist = Worklist {
            items: BTreeMap::new(),
            seq: 0,
            lifo: self.cex_mode && self.cex_depth_first,
        };
        let k0 = self.order_key(&init);
        worklist.push(k0, init);
        self.states = 0;
        while let Some((key, mut state)) = worklist.pop() {
            self.states += 1;
            // The state cap also guards the induction cases: a step case that
            // needs more states than this (ECA `Problem05`: 38 k in one
            // iteration) ends in a path-condition query that Z3 cannot finish
            // inside any timeout (measured: one 80 s `check` against a 2 s
            // timeout), so stopping early keeps the budget honest.
            if self.states > MAX_STATES {
                return Err(format!("budget-states{}", self.stats()));
            }
            // A fully concrete state (empty path condition) is not merged: an
            // `ite` over concrete siblings would only need the solver again at
            // the header enumeration, where a concrete footprint costs nothing.
            if state.parked {
                if !state.constraints.is_empty() || debug_flag("mergeall") {
                    let mk = merge_key(&state);
                    for other in worklist.take_mergeable(&key, &mk) {
                        let t0 = Instant::now();
                        state = self.merge(state, other);
                        self.t_merge += t0.elapsed();
                    }
                }
                if !state.restarted
                    && self.mode == Mode::Exhaustive
                    && self.fixpoint
                    && self.seen_header.len() < FIXPOINT_TOTAL_CAP
                    && self.loop_headers.contains(&(state.func, state.block))
                    && !self.fixpoint_off.contains(&(state.func, state.block))
                {
                    if let Some(restarts) = self.header_fixpoint(&mut state)? {
                        for s in restarts {
                            let k = self.order_key(&s);
                            worklist.push(k, s);
                        }
                        continue;
                    }
                }
                // Lazy modes: one path-condition query per loop iteration, on the
                // merged state (not on each arriving path).
                if self.lazy() && self.at_lazy_checkpoint(&state) && !self.path_alive(&mut state)? {
                    continue;
                }
            }
            self.step_state(state, &mut worklist)?;
        }
        Ok(())
    }

    /// Merge two states at the same program point (same key): the result's path
    /// condition is `pc_a ∨ pc_b`, a value both agree on is kept, a value they
    /// disagree on becomes `ite(pc_a, a, b)`, and anything not mergeable is
    /// dropped — a later read of it is then a fresh symbol (cells are marked
    /// escaped so a global's initialiser is NOT re-read), which only adds
    /// behaviours.
    // NOTE: one pass over every state component (values, cells, escaped and
    // zeroed sets, visits, constraints); splitting would scatter the invariant
    // that every component is merged under the same guard.
    #[allow(clippy::too_many_lines)]
    fn merge(&mut self, mut a: State, b: State) -> State {
        self.n_merges += 1;
        let t0 = Instant::now();
        // A fresh Boolean `g` selects the side. The constraints the sides share
        // (their common prefix: both descend from one fork) stay as they are;
        // the suffixes become `g → suffix_a` and `¬g → suffix_b`, and a merged
        // value `ite(g, x, y)`. The formula stays FLAT: nesting the whole path
        // condition in every merge (`ite(pc_a, x, y)`, `pc_a ∨ pc_b`) made one
        // query a 3,000-deep term that cost Z3 ~250 ms (minepump, 2026-10-10).
        let (guard, gv) = self.fresh_bool();
        let shared = a
            .constraints
            .iter()
            .zip(&b.constraints)
            .take_while(|(x, y)| x.0 == y.0)
            .count();
        let (pc_a, va) = conjunction(&a.constraints[shared..]);
        let (pc_b, vb) = conjunction(&b.constraints[shared..]);
        // One shared variable set for every value the sides disagree on: the
        // guard and the values' own. It over-approximates each value's
        // dependencies (only ever pulling more constraints into a query) and
        // keeps a merge linear instead of quadratic in the number of symbols.
        let mut vars = gv.clone();
        let keys: BTreeSet<ValueId> = a.values.keys().chain(b.values.keys()).copied().collect();
        let cells: BTreeSet<Cell> = a.memory.keys().chain(b.memory.keys()).cloned().collect();
        for k in &keys {
            if let (Some(Val::Bv(x, vx)), Some(Val::Bv(y, vy))) = (a.values.get(k), b.values.get(k))
            {
                if x != y {
                    vars = union(&vars, &union(vx, vy));
                }
            }
        }
        for k in &cells {
            if let (Some(Val::Bv(x, vx)), Some(Val::Bv(y, vy))) = (a.memory.get(k), b.memory.get(k))
            {
                if x != y {
                    vars = union(&vars, &union(vx, vy));
                }
            }
        }
        self.t_m_pc += t0.elapsed();
        for k in keys {
            let merged = match (a.values.get(&k), b.values.get(&k)) {
                (Some(Val::Bv(x, vx)), Some(Val::Bv(y, _))) if x.get_size() == y.get_size() => {
                    if x == y {
                        Some(Val::Bv(x.clone(), vx.clone()))
                    } else {
                        Some(Val::Bv(guard.ite(x, y), vars.clone()))
                    }
                }
                (Some(Val::Ptr(p)), Some(Val::Ptr(q))) if p == q => Some(Val::Ptr(p.clone())),
                _ => None,
            };
            if let Some(v) = merged {
                a.values.insert(k, v);
            } else {
                a.values.remove(&k);
            }
        }
        for k in cells {
            // A cell written on one side only still has a definite value on the
            // other: what a load there would have produced (the initialiser, or
            // any value for an uninitialised / escaped object). Materialising it
            // keeps the merge exact; dropping it to "escaped" made every later
            // read fresh and turned dead error paths live (product-lines: a
            // global flag set on one branch of an `if`).
            let (va, vb) = match (a.memory.get(&k).cloned(), b.memory.get(&k).cloned()) {
                (Some(x), Some(y)) => (x, y),
                (Some(x), None) => {
                    let y = self.unwritten_cell_value(&b, &k, &x);
                    (x, y)
                }
                (None, Some(y)) => {
                    let x = self.unwritten_cell_value(&a, &k, &y);
                    (x, y)
                }
                (None, None) => continue,
            };
            let merged = match (va, vb) {
                (Val::Bv(x, vx), Val::Bv(y, vy)) if x.get_size() == y.get_size() => {
                    if x == y {
                        Some(Val::Bv(x, vx))
                    } else {
                        // A cell present on one side only was not in the shared
                        // set: add both sides' symbols, or a later substitution
                        // of a pinned symbol would skip this cell (measured:
                        // a stale `s` in a global cell, hardness 2026-10-10).
                        let cell_vars = union(&vars, &union(&vx, &vy));
                        Some(Val::Bv(guard.ite(&x, &y), cell_vars))
                    }
                }
                (Val::Ptr(p), Val::Ptr(q)) => Some(Val::Ptr(if p == q { p } else { None })),
                _ => None,
            };
            if let Some(v) = merged {
                a.memory.insert(k, v);
            } else {
                a.memory.remove(&k);
                a.escaped.insert(k.0);
            }
        }
        a.escaped.extend(b.escaped);
        // Memory-safety bookkeeping: a block whose sides disagree is `Maybe`
        // (a later access abstains, the leak check counts it as unfreed), a dead
        // object stays dead, every alloca seen on either side was seen.
        let bases: BTreeSet<ValueId> = a.heap.keys().chain(b.heap.keys()).copied().collect();
        for base in bases {
            let merged = match (a.heap.get(&base), b.heap.get(&base)) {
                (Some(obj_a), Some(obj_b)) => {
                    let size = match (&obj_a.size, &obj_b.size) {
                        (Some((size_a, size_vars)), Some((size_b, _))) if size_a == size_b => {
                            Some((size_a.clone(), size_vars.clone()))
                        }
                        _ => None,
                    };
                    let status = if obj_a.status == obj_b.status {
                        obj_a.status
                    } else {
                        HeapStatus::Maybe
                    };
                    HeapObj { size, status }
                }
                (Some(only), None) | (None, Some(only)) => HeapObj {
                    size: only.size.clone(),
                    status: HeapStatus::Maybe,
                },
                (None, None) => continue,
            };
            a.heap.insert(base, merged);
        }
        a.dead.extend(b.dead);
        for entry in b.stack {
            if !a.stack.contains(&entry) {
                a.stack.push(entry);
            }
        }
        a.seen_alloca.extend(b.seen_alloca);
        a.versioned.extend(b.versioned);
        a.zeroed = a.zeroed.intersection(&b.zeroed).copied().collect();
        for (blk, n) in b.visits {
            let e = a.visits.entry(blk).or_insert(0);
            *e = (*e).max(n);
        }
        a.has_unknown |= b.has_unknown;
        let a_suffix = a.constraints.len() > shared;
        let b_suffix = b.constraints.len() > shared;
        a.constraints.truncate(shared);
        if a_suffix {
            a.constraints.push((guard.implies(&pc_a), union(&gv, &va)));
        }
        if b_suffix {
            a.constraints
                .push((guard.not().implies(&pc_b), union(&gv, &vb)));
        }
        a.checked_len = usize::MAX;
        a.pc_cache = None;
        a
    }

    /// The value a load of `cell` would produce in `st` when the cell was never
    /// written on that path: fresh for an escaped or uninitialised object, the
    /// global's initialiser at that path otherwise. `like` fixes the kind and
    /// width (a pointer cell's unwritten content is an opaque pointer).
    /// Header fixpoint (exhaustive pass): the state arriving at a loop header
    /// is replaced by the concrete states it stands for — every model of its
    /// footprint (values, cells, saved values) under the path condition,
    /// enumerated with blocking clauses — and only the ones not seen before are
    /// explored further, each as a constraint-free state. A loop whose header
    /// state ranges over a finite set thus terminates when no new state appears,
    /// however many iterations it runs (minepump: `while (1)` over a handful of
    /// small-domain globals). Exact: the continuation reads nothing outside the
    /// footprint, and the enumeration is complete. `Ok(None)`: this header's
    /// state is not finite enough (more than `FIXPOINT_CAP` models, or a solver
    /// UNKNOWN) — it is switched off for this header and the state is explored
    /// as before.
    /// The loop-header state sets of a completed lazy-pass proof (see
    /// [`HeaderStates`]): per header the fixpoint stayed on, every enumerated
    /// state projected to its integer SSA values and scalar globals. Headers the
    /// fixpoint gave up on, and everything when the total cap was hit, are left
    /// out (their sets would be incomplete).
    fn header_states(&self) -> Vec<HeaderStates> {
        if !self.fixpoint || self.seen_header.len() >= FIXPOINT_TOTAL_CAP {
            return Vec::new();
        }
        let mut by_header: BTreeMap<(FunctionId, BlockId), Vec<HeaderState>> = BTreeMap::new();
        for key in &self.seen_header {
            if self.fixpoint_off.contains(&(key.func, key.block)) {
                continue;
            }
            let mut slots = key.vals.iter();
            let mut state: HeaderState = Vec::new();
            for f in &key.frames {
                for _ in &f.5 {
                    slots.next();
                }
            }
            for vid in &key.value_ids {
                if let Some(CVal::Bv(v, bits)) = slots.next() {
                    state.push((StateVar::Value(*vid), *v, *bits));
                }
            }
            let mut written: BTreeSet<ValueId> = BTreeSet::new();
            for cell in &key.cells {
                let slot = slots.next();
                if cell.1.is_empty() && self.globals.contains_key(&cell.0) {
                    written.insert(cell.0);
                    if let Some(CVal::Bv(v, bits)) = slot {
                        state.push((StateVar::Global(cell.0), *v, *bits));
                    }
                }
            }
            // A scalar global never written on the path (and not escaped) still
            // holds its initialiser there — part of the state all the same.
            for (gid, (init, is_const)) in &self.globals {
                if *is_const || written.contains(gid) || key.escaped.contains(gid) {
                    continue;
                }
                let scalar = match init {
                    Some(Constant::Int { value, bits }) => {
                        let bits = u32::from(*bits);
                        (bits > 0 && bits <= 64).then(|| {
                            let mask = if bits == 64 {
                                u64::MAX
                            } else {
                                (1u64 << bits) - 1
                            };
                            // INVARIANT: the bit pattern of a `bits`-wide value.
                            #[allow(clippy::cast_sign_loss)]
                            let v = (*value as u64) & mask;
                            (v, bits)
                        })
                    }
                    Some(Constant::ZeroInit) => self
                        .module
                        .globals
                        .iter()
                        .find(|g| g.id == *gid)
                        .and_then(|g| g.value_type)
                        .and_then(|t| match self.module.types.get(&t) {
                            Some(saf_core::air::AirType::Integer { bits }) => {
                                Some((0u64, u32::from(*bits)))
                            }
                            _ => None,
                        }),
                    _ => None,
                };
                if let Some((v, bits)) = scalar {
                    state.push((StateVar::Global(*gid), v, bits));
                }
            }
            by_header
                .entry((key.func, key.block))
                .or_default()
                .push(state);
        }
        by_header
            .into_iter()
            .map(|((func, block), states)| HeaderStates {
                func,
                block,
                states,
            })
            .collect()
    }

    fn header_fixpoint(&mut self, st: &mut State) -> Res<Option<Vec<State>>> {
        let (mut key, syms) = footprint(st);
        let mut keys: Vec<ConcreteKey> = Vec::new();
        if syms.is_empty() {
            if self.lazy() && !self.path_alive(st)? {
                return Ok(Some(Vec::new())); // a dead path
            }
            keys.push(key);
        } else {
            let (pc, _) = path_condition(st);
            self.solver.reset();
            self.solver.assert(&pc);
            loop {
                if Instant::now() > self.deadline {
                    return Err(format!("budget-time{}", self.stats()));
                }
                if keys.len() >= FIXPOINT_CAP {
                    trace(|| format!("fixpoint off: {:?} has > {FIXPOINT_CAP} states", st.block));
                    self.fixpoint_off.insert((st.func, st.block));
                    return Ok(None);
                }
                self.solver_calls += 1;
                let t0 = Instant::now();
                let res = self.solver.check();
                self.t_check += t0.elapsed();
                match res {
                    SatResult::Unsat => break,
                    SatResult::Unknown => {
                        self.fixpoint_off.insert((st.func, st.block));
                        return Ok(None);
                    }
                    SatResult::Sat => {
                        let model = self.solver.get_model().ok_or("fixpoint-no-model")?;
                        let mut lits: Vec<Bool> = Vec::with_capacity(syms.len());
                        for (slot, bv) in &syms {
                            let v = model
                                .eval(bv, true)
                                .and_then(|b| b.as_u64())
                                .ok_or("fixpoint-model-eval")?;
                            key.vals[*slot] = CVal::Bv(v, bv.get_size());
                            lits.push(eq(bv, &BV::from_u64(v, bv.get_size())));
                        }
                        keys.push(key.clone());
                        let blocking: Vec<&Bool> = lits.iter().collect();
                        self.solver.assert(Bool::and(&blocking).not());
                    }
                }
            }
        }
        trace(|| {
            format!(
                "fixpoint: {:?} visits={} -> {} concrete states ({} symbolic slots)",
                st.block,
                st.visits.get(&st.block).copied().unwrap_or(0),
                keys.len(),
                syms.len()
            )
        });
        let mut out = Vec::new();
        for k in keys {
            let new = self.seen_header.insert(k.clone());
            trace(|| format!("fixpoint key new={new} vals={:?}", k.vals));
            if new {
                out.push(restart_state(st, &k));
            }
        }
        self.n_restarts += out.len() as u64;
        Ok(Some(out))
    }

    fn unwritten_cell_value(&mut self, st: &State, cell: &Cell, like: &Val) -> Val {
        let Val::Bv(x, _) = like else {
            return Val::Ptr(None);
        };
        let bits = x.get_size();
        if !st.escaped.contains(&cell.0) {
            if let Some((c, size)) = self.init_scalar(cell) {
                if size.saturating_mul(8) == u64::from(bits) {
                    if let Ok(v @ Val::Bv(..)) = self.constant(&c, Some(bits)) {
                        if matches!(&v, Val::Bv(b, _) if b.get_size() == bits) {
                            return v;
                        }
                    }
                }
            }
        }
        self.fresh_val(bits)
    }

    /// Drop the values that cannot be read after the current block.
    fn prune_block_locals(&self, st: &mut State) {
        if debug_flag("noprune") {
            return;
        }
        if let Some(locals) = self.block_local.get(&st.block) {
            for v in locals {
                st.values.remove(v);
            }
        }
    }

    /// Counters appended to a budget abstain, for the probes.
    fn stats(&self) -> String {
        format!(
            "({}:steps={},solver={},states={},restarts={},simplify={}ms,check={}ms,merge={}ms/{}n/pc{}ms)",
            match self.mode {
                Mode::Exhaustive if self.lazy_exhaustive => "lexh".to_string(),
                Mode::Exhaustive => "exh".to_string(),
                Mode::Base { k } => format!("base{k}"),
                Mode::Step { k } => format!("step{k}"),
            },
            self.steps,
            self.solver_calls,
            self.states,
            self.n_restarts,
            self.t_simplify.as_millis(),
            self.t_check.as_millis(),
            self.t_merge.as_millis(),
            self.n_merges,
            self.t_m_pc.as_millis()
        )
    }

    /// Execute one state until it terminates or forks; successors go to `worklist`.
    /// Program-order key of a state: loop iteration first, then the
    /// reverse-postorder position of each call site on the stack and of the
    /// current block; a trailing `usize::MAX` puts a caller that has returned
    /// from a call after every state still inside it.
    fn order_key(&self, st: &State) -> Vec<usize> {
        let iteration = self
            .kloop
            .as_ref()
            .map_or(0, |lp| st.visits.get(&lp.header).copied().unwrap_or(0));
        let mut key: Vec<usize> = Vec::with_capacity(st.frames.len() + 4);
        key.push(iteration as usize);
        // Then by how often the block was entered, so one loop iteration
        // completes (and merges) before the next starts; RPO alone popped the
        // header as soon as one path reached it, defeating the merge.
        key.push(st.visits.get(&st.block).copied().unwrap_or(0) as usize);
        for fr in &st.frames {
            key.push(
                self.rpo
                    .get(&(fr.func, fr.ret_block))
                    .copied()
                    .unwrap_or(usize::MAX - 1),
            );
        }
        key.push(
            self.rpo
                .get(&(st.func, st.block))
                .copied()
                .unwrap_or(usize::MAX - 1),
        );
        key.push(usize::MAX);
        key
    }

    // NOTE: the executor's main loop — entry bookkeeping, parking, and the
    // control-flow dispatch belong together; splitting them would hide the
    // order the soundness argument of §8 depends on.
    #[allow(clippy::too_many_lines)]
    fn step_state(&mut self, mut st: State, worklist: &mut Worklist) -> Res<()> {
        loop {
            if Instant::now() > self.deadline {
                return Err(format!("budget-time{}", self.stats()));
            }
            self.steps += 1;
            if self.steps > MAX_STEPS {
                return Err(format!("budget-steps{}", self.stats()));
            }
            let block = self.block(st.func, st.block)?;
            // Block-entry bookkeeping, once per entry: a state parked at a join
            // block without phis is resumed at index 0 too (`parked` is still
            // set then), and counting that entry twice made the induction cases
            // stop an iteration early — a vacuous, WRONG proof (2026-10-09).
            if st.idx == 0 && !st.parked {
                st.restarted = false;
                if self.cex_mode {
                    st.path.push((st.func, st.block));
                }
                let v = st.visits.entry(st.block).or_insert(0);
                *v += 1;
                if *v > VISIT_CAP {
                    return Err("unwind-cap".to_string());
                }
                if let Some(stop) = self.loop_bookkeeping(&st) {
                    trace(|| {
                        format!(
                            "bookkeeping stop at block={:?} visits={:?} mode={:?}",
                            st.block, st.visits, self.mode
                        )
                    });
                    return stop;
                }
                // Lazy modes: kill a dead path once per loop iteration.
                // At a join block (every loop header is one) the check runs once
                // on the merged state instead (see `run`).
                if self.lazy()
                    && self.at_lazy_checkpoint(&st)
                    && !self.join_blocks.contains(&st.block)
                    && !self.path_alive(&mut st)?
                {
                    return Ok(());
                }
            }
            let Some(inst) = block.instructions.get(st.idx) else {
                return Err("block-without-terminator".to_string());
            };
            // Park once per block entry at a join point, after its phis — except
            // a state with an EMPTY path condition (fully concrete after a
            // domain split or fixpoint restart) at an ordinary join: merging it
            // with a sibling would only turn its concrete values into `ite`s
            // and every later branch back into a query. Loop headers still park
            // (the header fixpoint acts on parked states). An ECA iteration is
            // then six straight concrete runs instead of 12 k parked states.
            if !st.parked
                && !self.cex_mode
                && !matches!(inst.op, Operation::Phi { .. })
                && self.join_blocks.contains(&st.block)
                && !st.parked_here
                && (!st.constraints.is_empty() || self.loop_headers.contains(&(st.func, st.block)))
            {
                st.parked = true;
                st.parked_here = true;
                let key = self.order_key(&st);
                worklist.push(key, st);
                return Ok(());
            }
            st.parked = false;
            let flow = self.exec(&mut st, inst).map_err(|e| {
                trace(|| format!("abstain `{e}` at {:?}", inst.op));
                e
            })?;
            match flow {
                Flow::Next => st.idx += 1,
                Flow::Jump(target) => {
                    st.prev = Some(st.block);
                    st.block = target;
                    st.idx = 0;
                    st.parked_here = false;
                }
                Flow::Refine(conds) => {
                    trace(|| format!("refine at block={:?} into {} states", st.block, conds.len()));
                    for (cond, vars) in conds {
                        let mut s = st.clone();
                        Self::take_branch(&mut s, cond, vars);
                        // Same instruction, now with the symbol pinned.
                        let key = self.order_key(&s);
                        worklist.push(key, s);
                    }
                    return Ok(());
                }
                Flow::Fork(mut targets) => {
                    trace(|| format!("fork at block={:?} -> {} targets", st.block, targets.len()));
                    // One live side: just go there (no clone, no queue — a
                    // concrete ECA iteration is a thousand such "forks").
                    if targets.len() == 1 {
                        let (target, cond, vars) = targets.pop().ok_or("fork-empty")?;
                        Self::take_branch(&mut st, cond, vars);
                        st.prev = Some(st.block);
                        st.block = target;
                        st.idx = 0;
                        st.parked_here = false;
                        continue;
                    }
                    for (target, cond, vars) in targets {
                        let mut s = st.clone();
                        Self::take_branch(&mut s, cond, vars);
                        s.prev = Some(st.block);
                        s.block = target;
                        s.idx = 0;
                        s.parked_here = false;
                        let key = self.order_key(&s);
                        worklist.push(key, s);
                    }
                    return Ok(());
                }
                Flow::Return(val) => {
                    if let (Mode::Step { k }, Some(lp)) = (self.mode, self.kloop.as_ref()) {
                        // Returning from the loop's function within the first k
                        // iterations: the path leaves the loop, irrelevant to the step.
                        if st.func == lp.func
                            && st.frames.len() == usize::from(lp.call_site.is_some())
                            && st.visits.get(&lp.header).copied().unwrap_or(0) <= k
                        {
                            return Ok(());
                        }
                    }
                    if self.goal == Goal::MemSafety {
                        let depth = st.frames.len();
                        let (dying, live): (Vec<_>, Vec<_>) =
                            st.stack.iter().partition(|(d, _)| *d == depth);
                        st.dead.extend(dying.into_iter().map(|(_, b)| b));
                        st.stack = live;
                        if depth == 0 {
                            self.ms_memtrack(&st)?;
                        }
                    }
                    let Some(frame) = st.frames.pop() else {
                        return Ok(()); // `main` returned: this path is safe.
                    };
                    for (vid, old) in frame.saved {
                        match old {
                            Some(v) => {
                                st.values.insert(vid, v);
                            }
                            None => {
                                st.values.remove(&vid);
                            }
                        }
                    }
                    match (frame.ret_dst, val) {
                        (Some(dst), Some(v)) => {
                            st.values.insert(dst, v);
                        }
                        // No value came back (`return undef`, or a void callee
                        // whose call still has a result id): any value of the
                        // call's type, or nothing when the type is unknown.
                        (Some(dst), None) => match self.width.get(&dst) {
                            Some(&w) => {
                                let v = self.fresh_val(w);
                                st.values.insert(dst, v);
                            }
                            None if self.is_ptr.contains(&dst) => {
                                st.values.insert(dst, Val::Ptr(None));
                            }
                            None => {}
                        },
                        (None, _) => {}
                    }
                    st.func = frame.func;
                    st.block = frame.ret_block;
                    st.idx = frame.ret_idx + 1;
                    st.prev = frame.ret_prev;
                    st.parked_here = true; // mid-block: no join here
                }
                Flow::Halt => {
                    trace(|| format!("halt at block={:?}", st.block));
                    if self.goal == Goal::MemSafety && !st.dead_end {
                        self.ms_memtrack(&st)?;
                    }
                    return Ok(());
                }
            }
        }
    }

    /// k-induction bookkeeping at a block entry in `main` (plan 223). `Some(r)`
    /// ends the path with `r`; `None` continues.
    fn loop_bookkeeping(&self, st: &State) -> Option<Res<()>> {
        let lp = self.kloop.as_ref()?;
        let depth = usize::from(lp.call_site.is_some());
        if st.frames.len() != depth || st.func != lp.func {
            return None;
        }
        let iteration = st.visits.get(&lp.header).copied().unwrap_or(0);
        match self.mode {
            Mode::Exhaustive => None,
            Mode::Base { k } => {
                // Iterations beyond the base case belong to the step case.
                (st.block == lp.header && iteration > k).then_some(Ok(()))
            }
            Mode::Step { k } => {
                if st.block == lp.header {
                    // Iteration k + 1 finished without error: the step holds here.
                    return (iteration > k + 1).then_some(Ok(()));
                }
                // Leaving the loop during the assumed iterations: a path that never
                // reaches iteration k + 1, irrelevant to the inductive step.
                (!lp.body.contains(&st.block) && iteration <= k).then_some(Ok(()))
            }
        }
    }

    // ---- values ----------------------------------------------------------

    fn val(&mut self, st: &mut State, vid: ValueId, want_bits: Option<u32>) -> Res<Val> {
        if let Some(v) = st.values.get(&vid) {
            return Ok(v.clone());
        }
        if self.globals.contains_key(&vid) {
            return Ok(Val::Ptr(Some((vid, Vec::new()))));
        }
        if self.opaque_globals.contains(&vid) {
            return Ok(Val::Ptr(None));
        }
        if let Some(c) = self.module.constants.get(&vid) {
            let c = c.clone();
            let v = self
                .constant(&c, want_bits.or_else(|| self.width.get(&vid).copied()))
                .map_err(|e| {
                    trace(|| format!("constant {vid:?} ({}) -> {e}", self.describe(vid)));
                    e
                })?;
            st.values.insert(vid, v.clone());
            return Ok(v);
        }
        // No definition anywhere: an `undef`/`poison` operand (an uninitialised
        // local read before assignment, post-mem2reg) or a value the frontend
        // left opaque. Admitting EVERY value of its width only adds behaviours,
        // so a proof over it stays sound; without a width there is no safe
        // symbol to mint.
        if let Some(bits) = want_bits.or_else(|| self.width.get(&vid).copied()) {
            trace(|| format!("fresh: undefined operand {vid:?} ({bits} bits)"));
            let v = self.fresh_val(bits);
            st.values.insert(vid, v.clone());
            return Ok(v);
        }
        if self.is_ptr.contains(&vid) {
            return Ok(Val::Ptr(None));
        }
        // A function used as a value (`ops->probe = f`): a KNOWN pointer whose
        // base is the function's own id, so an indirect call through it (after
        // any number of stores and loads) resolves to that function.
        if self
            .module
            .functions
            .iter()
            .any(|f| f.id.raw() == vid.raw())
        {
            trace(|| format!("function address {vid:?}"));
            return Ok(Val::Ptr(Some((vid, Vec::new()))));
        }
        trace(|| format!("undefined-value: {vid:?} defined by {}", self.describe(vid)));
        Err("undefined-value".to_string())
    }

    /// Developer trace helper: the instruction defining `vid`, or what else it is.
    fn describe(&self, vid: ValueId) -> String {
        if let Some(c) = self.module.constants.get(&vid) {
            return format!("constant {c:?}");
        }
        if self.module.globals.iter().any(|g| g.id == vid) {
            return "global".to_string();
        }
        for f in &self.module.functions {
            if f.params.iter().any(|p| p.id == vid) {
                return format!("param of {}", f.name);
            }
            for b in &f.blocks {
                if let Some(i) = b.instructions.iter().find(|i| i.dst == Some(vid)) {
                    return format!("{:?} in {} (result_type {:?})", i.op, f.name, i.result_type);
                }
            }
        }
        "nothing in the module".to_string()
    }

    fn constant(&mut self, c: &Constant, want_bits: Option<u32>) -> Res<Val> {
        Ok(match c {
            Constant::Int { value, bits } => {
                let bits = u32::from(*bits);
                if bits == 0 || bits > 64 {
                    return Err("const-width".to_string());
                }
                Val::Bv(int_bv(*value, bits), no_vars())
            }
            Constant::Null => Val::Ptr(Some((NULL_BASE, Vec::new()))),
            // A wide integer (also how the frontend carries an x87 `long
            // double` constant: its exact 80-bit pattern).
            Constant::BigInt { value, bits } => {
                let bits = u32::from(*bits);
                let v: i128 = value.parse().map_err(|_| "const-bigint".to_string())?;
                if bits == 0 || bits > 128 {
                    return Err("const-width".to_string());
                }
                // INVARIANT: the halves are taken from a 128-bit value by shifting.
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let (hi, lo) = ((v >> 64) as u64, v as u64);
                let lo_bv = int_bv(i64::from_ne_bytes(lo.to_ne_bytes()), 64);
                if bits <= 64 {
                    Val::Bv(lo_bv.extract(bits - 1, 0), no_vars())
                } else {
                    let hi_bv =
                        int_bv(i64::from_ne_bytes(hi.to_ne_bytes()), 64).extract(bits - 65, 0);
                    Val::Bv(hi_bv.concat(&lo_bv), no_vars())
                }
            }
            // IEEE encoding of the (f64-stored) constant at the type's width.
            Constant::Float { value, bits } => match bits {
                // INVARIANT: a 32-bit float constant is stored losslessly in the
                // f64 and comes back exactly from this narrowing.
                #[allow(clippy::cast_possible_truncation)]
                32 => Val::Bv(
                    int_bv(i64::from(f32::to_bits(*value as f32)), 32),
                    no_vars(),
                ),
                64 => Val::Bv(
                    int_bv(i64::from_ne_bytes(value.to_bits().to_ne_bytes()), 64),
                    no_vars(),
                ),
                _ => return Err("const-float-width".to_string()),
            },
            Constant::ZeroInit => match want_bits {
                Some(b) => Val::Bv(int_bv(0, b), no_vars()),
                None => Val::Ptr(None),
            },
            Constant::Undef => match want_bits {
                Some(b) => self.fresh_val(b),
                None => Val::Ptr(None),
            },
            Constant::GlobalRef(g) => {
                if self.globals.contains_key(g) {
                    Val::Ptr(Some((*g, Vec::new())))
                } else {
                    Val::Ptr(None)
                }
            }
            // `&g.f`: an exact cell of a modelled global; opaque otherwise.
            Constant::GlobalElem { base, path } => {
                if self.globals.contains_key(base) {
                    let cell = self
                        .layouts
                        .get(base)
                        .and_then(|l| l.offset_of_path(path))
                        .and_then(|o| u32::try_from(o).ok())
                        .filter(|o| *o < IMPRECISE)
                        .map_or((*base, vec![IMPRECISE]), |o| cell_at(*base, o));
                    Val::Ptr(Some(cell))
                } else {
                    Val::Ptr(None)
                }
            }
            other => {
                trace(|| format!("const-unmodelled: {other:?}"));
                return Err("const-unmodelled".to_string());
            }
        })
    }

    fn bv(&mut self, st: &mut State, vid: ValueId) -> Res<(BV, Vars)> {
        let want = self.width.get(&vid).copied();
        match self.val(st, vid, want)? {
            Val::Bv(b, vs) => Ok((b, vs)),
            Val::Ptr(_) => Err("pointer-as-int".to_string()),
        }
    }

    /// Wrap a computed bitvector: concrete results are simplified to literals so
    /// a long concrete path (a 1024-iteration counter) stays a constant, not a
    /// 1024-deep term.
    fn mk(bv: BV, vars: Vars) -> Val {
        if vars.is_empty() {
            Val::Bv(bv.simplify(), vars)
        } else {
            Val::Bv(bv, vars)
        }
    }

    fn ptr(&mut self, st: &mut State, vid: ValueId) -> Res<Option<Cell>> {
        match self.val(st, vid, None)? {
            Val::Ptr(p) => Ok(p),
            Val::Bv(..) => {
                trace(|| format!("int-as-pointer: {vid:?} is {}", self.describe(vid)));
                Err("int-as-pointer".to_string())
            }
        }
    }

    fn result_bits(&self, inst: &Instruction) -> Res<u32> {
        inst.dst
            .and_then(|d| self.width.get(&d).copied())
            .ok_or_else(|| "result-width".to_string())
    }

    // ---- execution -------------------------------------------------------

    // NOTE: one arm per AIR operation, in one match, is the whole point of this
    // function: the soundness argument is "every operation is either modelled
    // exactly here or abstains here", which a reader must be able to audit in one
    // place. Splitting by operation family would scatter that invariant.
    #[allow(clippy::too_many_lines)]
    fn exec(&mut self, st: &mut State, inst: &Instruction) -> Res<Flow> {
        let set = |st: &mut State, inst: &Instruction, v: Val| {
            if let Some(d) = inst.dst {
                st.values.insert(d, v);
            }
        };
        match &inst.op {
            Operation::BinaryOp { kind } => {
                let [a_id, b_id] = inst.operands[..] else {
                    return Err("binop-arity".to_string());
                };
                // Pointer (in)equality: two modelled cells compare by identity; a
                // comparison involving an opaque pointer admits both outcomes.
                if matches!(kind, BinaryOp::ICmpEq | BinaryOp::ICmpNe)
                    && (self.is_ptr.contains(&a_id) || self.is_ptr.contains(&b_id))
                {
                    let pa = self.ptr(st, a_id)?;
                    let pb = self.ptr(st, b_id)?;
                    let v = match (pa, pb) {
                        (Some(x), Some(y)) if !is_imprecise(&x) && !is_imprecise(&y) => {
                            let same = x == y;
                            let truth = if matches!(kind, BinaryOp::ICmpEq) {
                                same
                            } else {
                                !same
                            };
                            Val::Bv(int_bv(i64::from(truth), 1), no_vars())
                        }
                        _ => self.fresh_val(1),
                    };
                    set(st, inst, v);
                    return Ok(Flow::Next);
                }
                let wa = self.width.get(&a_id).copied();
                let wb = self.width.get(&b_id).copied();
                let (a, va) = match self.val(st, a_id, wa.or(wb))? {
                    Val::Bv(x, vx) => (x, vx),
                    Val::Ptr(_) => return Err("pointer-as-int".to_string()),
                };
                let (b, vb) = match self.val(st, b_id, wb.or(Some(a.get_size())))? {
                    Val::Bv(x, vx) => (x, vx),
                    Val::Ptr(_) => return Err("pointer-as-int".to_string()),
                };
                if a.get_size() != b.get_size() {
                    return Err("binop-width-mismatch".to_string());
                }
                let vars = union(&va, &vb);
                // Counterexample mode leaves `shl` alone: the replay confirms
                // only arithmetic overflow (UBSan's `signed-integer-overflow`),
                // and the IR does not say whether a shift was signed at all.
                let unsigned_shl = *kind == BinaryOp::Shl && self.shl_is_unsigned(inst);
                if self.goal == Goal::SignedOverflow
                    && !unsigned_shl
                    && !(self.cex_mode && !self.cex_check_shl && *kind == BinaryOp::Shl)
                {
                    if let Some(ovf) = overflow_condition(inst, *kind, &a, &b) {
                        trace(|| format!("overflow check: {kind:?} at {:?}", inst.id));
                        self.check_violation(st, Some((&ovf, &vars)), inst.id)?;
                    }
                }
                let v = self.binop(st, *kind, &a, &b, &vars)?;
                set(st, inst, Self::mk(v, vars));
                Ok(Flow::Next)
            }
            Operation::Cast { kind, target_bits } => {
                let src = inst.operands.first().copied().ok_or("cast-arity")?;
                let v = match kind {
                    CastKind::Trunc | CastKind::ZExt | CastKind::SExt => {
                        let to = match target_bits {
                            Some(b) => u32::from(*b),
                            None => self.result_bits(inst)?,
                        };
                        let (a, vars) = match self.bv(st, src) {
                            Ok(x) => x,
                            // A cast of `undef` (its own width unrecorded): any
                            // value of the result width.
                            Err(e) if e == "undefined-value" => {
                                set(st, inst, self.fresh_val(to));
                                return Ok(Flow::Next);
                            }
                            Err(e) => return Err(e),
                        };
                        let from = a.get_size();
                        let r = match kind {
                            CastKind::Trunc if to <= from => a.extract(to - 1, 0),
                            CastKind::ZExt if to >= from => a.zero_ext(to - from),
                            CastKind::SExt if to >= from => a.sign_ext(to - from),
                            _ => return Err("cast-width".to_string()),
                        };
                        Self::mk(r, vars)
                    }
                    CastKind::Bitcast => {
                        // Pointer-to-pointer bitcasts keep identity; int bitcasts of
                        // equal width are the identity too.
                        let v = self.val(st, src, self.width.get(&src).copied())?;
                        match (&v, inst.dst.map(|d| self.is_ptr.contains(&d))) {
                            (Val::Ptr(_), Some(true)) => v,
                            (Val::Bv(b, _), Some(false))
                                if Some(b.get_size()) == self.result_bits(inst).ok() =>
                            {
                                v
                            }
                            _ => return Err("bitcast-unmodelled".to_string()),
                        }
                    }
                    // A pointer's integer image is unknown to the engine (no
                    // address model): any value. Sound because the way back,
                    // `inttoptr`, yields an opaque pointer (stores abstain).
                    CastKind::PtrToInt => {
                        let bits = self.result_bits(inst)?;
                        trace(|| "fresh: ptrtoint".to_string());
                        self.fresh_val(bits)
                    }
                    // An integer turned pointer may point anywhere: opaque (a
                    // store through it abstains, a load is fresh).
                    CastKind::IntToPtr => Val::Ptr(None),
                    CastKind::SIToFP | CastKind::UIToFP => {
                        let (a, vars) = self.bv(st, src)?;
                        let to = self.result_bits(inst)?;
                        Self::mk(fpa::from_int(&a, *kind == CastKind::SIToFP, to)?, vars)
                    }
                    CastKind::FPExt | CastKind::FPTrunc => {
                        let (a, vars) = self.bv(st, src)?;
                        let to = self.result_bits(inst)?;
                        Self::mk(fpa::convert(&a, to)?, vars)
                    }
                    // Float to integer truncates toward zero; a NaN or a value
                    // outside the integer's range is undefined behaviour in C
                    // and unspecified in SMT-LIB, so the result is then ANY
                    // value of the width (never one Z3 happens to pick).
                    CastKind::FPToSI | CastKind::FPToUI => {
                        let (a, vars) = self.bv(st, src)?;
                        let to = self.result_bits(inst)?;
                        let (conv, in_range) = fpa::to_int(&a, *kind == CastKind::FPToSI, to)?;
                        let (any, avars) = self.fresh(to);
                        Val::Bv(in_range.ite(&conv, &any), union(&vars, &avars))
                    }
                    CastKind::AddrSpaceCast => return Err(format!("cast-{kind:?}")),
                };
                set(st, inst, v);
                Ok(Flow::Next)
            }
            Operation::Copy | Operation::Freeze => {
                let src = inst.operands.first().copied().ok_or("copy-arity")?;
                let v = self.val(st, src, self.width.get(&src).copied())?;
                set(st, inst, v);
                Ok(Flow::Next)
            }
            Operation::Select => {
                let [cond_id, lhs_id, rhs_id] = inst.operands[..] else {
                    return Err("select-arity".to_string());
                };
                let (cond_bv, cond_vars) = self.bv(st, cond_id)?;
                let cond = truthy(&cond_bv);
                let lhs = self.val(st, lhs_id, self.width.get(&lhs_id).copied())?;
                let rhs = self.val(st, rhs_id, self.width.get(&rhs_id).copied())?;
                let v = match (lhs, rhs) {
                    (Val::Bv(lb, lv), Val::Bv(rb, rv)) if lb.get_size() == rb.get_size() => {
                        let vars = union(&cond_vars, &union(&lv, &rv));
                        Self::mk(cond.ite(&lb, &rb), vars)
                    }
                    (Val::Ptr(lp), Val::Ptr(rp)) if lp == rp => Val::Ptr(lp),
                    _ => return Err("select-unmodelled".to_string()),
                };
                set(st, inst, v);
                Ok(Flow::Next)
            }
            // The phis of a block execute SIMULTANEOUSLY: every operand is read
            // in the state at block entry, before any phi of the block is
            // assigned. Reading them one by one let `b = phi [.., a]` see the
            // new `a` — a swap became a copy, and a deterministic btor2c
            // state machine was "proven" safe (`dyn_partition`, 2026-10-10).
            // All phis are evaluated at the first one; the others are no-ops.
            Operation::Phi { .. } => {
                if st.idx == 0 {
                    let prev = st.prev.ok_or("phi-without-pred")?;
                    let block = self.block(st.func, st.block)?;
                    let mut assigned: Vec<(ValueId, Val)> = Vec::new();
                    for phi in block
                        .instructions
                        .iter()
                        .take_while(|i| matches!(i.op, Operation::Phi { .. }))
                    {
                        let Operation::Phi { incoming } = &phi.op else {
                            unreachable!("take_while keeps phis only");
                        };
                        let (_, src) = incoming
                            .iter()
                            .find(|(b, _)| *b == prev)
                            .ok_or("phi-missing-edge")?;
                        let want = phi.dst.and_then(|d| self.width.get(&d).copied());
                        let v = match self.val(st, *src, want) {
                            Ok(v) => v,
                            // An `undef` incoming of a pointer phi: any pointer.
                            Err(e)
                                if e == "undefined-value"
                                    && phi.dst.is_some_and(|d| self.is_ptr.contains(&d)) =>
                            {
                                Val::Ptr(None)
                            }
                            Err(e) => return Err(e),
                        };
                        if let Some(d) = phi.dst {
                            assigned.push((d, v));
                        }
                    }
                    for (d, v) in assigned {
                        st.values.insert(d, v);
                    }
                }
                Ok(Flow::Next)
            }
            Operation::Alloca { .. } => {
                let d = inst.dst.ok_or("alloca-dst")?;
                if self.goal == Goal::MemSafety {
                    // The first execution on a path is the object `d`; a
                    // re-execution (a second call, a loop) is a FRESH object, so
                    // a pointer kept from an earlier activation never aliases
                    // the new one (it stays dead).
                    let base = if st.seen_alloca.insert(d) {
                        d
                    } else {
                        self.alloca_versions += 1;
                        let b = ValueId::new(u128::MAX / 2 - u128::from(self.alloca_versions));
                        st.versioned.insert(b, d);
                        b
                    };
                    st.dead.remove(&base);
                    st.stack.push((st.frames.len(), base));
                    st.memory.retain(|k, _| k.0 != base);
                    st.escaped.remove(&base);
                    set(st, inst, Val::Ptr(Some((base, Vec::new()))));
                    return Ok(Flow::Next);
                }
                // A fresh object: a previous iteration's cells are dead.
                st.memory.retain(|k, _| k.0 != d);
                st.escaped.remove(&d);
                set(st, inst, Val::Ptr(Some((d, Vec::new()))));
                Ok(Flow::Next)
            }
            // A heap allocation is a fresh object of its own: cells never written
            // read as any value (indeterminate contents — `calloc`'s zeros are
            // within that), its base is unique per execution of the allocation so
            // two live objects never alias, and nothing frees it (a later access
            // to freed memory is undefined behaviour the engine need not model).
            Operation::HeapAlloc { kind } => {
                self.fresh_counter += 1;
                let base = ValueId::new(u128::MAX - u128::from(self.fresh_counter));
                let zeroing = match kind {
                    saf_core::air::HeapAllocKind::Calloc => true,
                    saf_core::air::HeapAllocKind::Other(name) => {
                        matches!(name.as_str(), "kzalloc" | "vzalloc" | "kmem_cache_zalloc")
                    }
                    _ => false,
                };
                if self.goal == Goal::MemSafety {
                    let size = self.ms_alloc_size(st, inst, kind)?;
                    st.heap.insert(
                        base,
                        HeapObj {
                            size: Some(size),
                            status: HeapStatus::Live,
                        },
                    );
                }
                if zeroing {
                    st.zeroed.insert(base);
                }
                trace(|| format!("heap object {base:?} zeroed={zeroing}"));
                set(st, inst, Val::Ptr(Some((base, Vec::new()))));
                Ok(Flow::Next)
            }
            // A GEP with all-constant indices on a modelled pointer names another
            // modelled cell: the path grows by the indices. An instruction GEP's
            // first index is pointer-level (`p + i * sizeof(*p)`), not a descent,
            // and must be 0 to stay inside the object; the frontend's synthetic
            // GEPs for constant expressions on globals (recognisable by carrying
            // no source span) already dropped that index. Anything else — a
            // dynamic index, a non-zero pointer-level step, an opaque base — is an
            // opaque pointer. No cell is invalidated here: an opaque pointer can
            // only be read through (fresh) or stored through (abstain).
            // A pointer `base + Σ steps`, by the GEP's byte arithmetic. Cells are
            // byte offsets into an object, never structural paths: the GEP's
            // `FieldPath` is relative to the GEP's own source element type, which
            // need not be the shape of the object (`float (*row)[14]` over a
            // `[32 x [14 x float]]` global read `W[0][i]` as `W[i][0]` — the
            // wrong-TRUE on `neural-networks/gcas_6_unsafe`, 2026-10-11).
            Operation::Gep { .. } => {
                let base_id = inst.operands.first().copied().ok_or("gep-arity")?;
                let v = match self.ptr(st, base_id)? {
                    Some((b, path)) => {
                        let steps = self.gep_bytes.get(&inst.id).cloned();
                        let cell = match (cell_offset(&(b, path)), steps) {
                            (Some(off0), Some(steps)) if steps.len() + 1 == inst.operands.len() => {
                                let mut off = i128::from(off0);
                                let mut exact = true;
                                for (i, (is_field, n)) in steps.iter().enumerate() {
                                    if *is_field {
                                        off += i128::from(*n);
                                        continue;
                                    }
                                    // An index that is a constant ON THIS PATH
                                    // (`a[i]` with `i` concrete after a fixpoint
                                    // restart or in an unrolled counting loop)
                                    // names an exact cell; a symbolic one leaves
                                    // the offset unknown.
                                    let op = inst.operands[i + 1];
                                    let k = if debug_flag("noconcidx") {
                                        None
                                    } else {
                                        match self.val(st, op, self.width.get(&op).copied())? {
                                            Val::Bv(b, _) => b.simplify().as_i64(),
                                            Val::Ptr(_) => None,
                                        }
                                    };
                                    let Some(k) = k else {
                                        exact = false;
                                        break;
                                    };
                                    off += i128::from(k) * i128::from(*n);
                                }
                                match u32::try_from(off) {
                                    Ok(o) if exact && o < IMPRECISE => cell_at(b, o),
                                    // Still inside `b` (a pointer leaving its
                                    // object is undefined behaviour), at an
                                    // unknown offset.
                                    _ => (b, vec![IMPRECISE]),
                                }
                            }
                            _ => (b, vec![IMPRECISE]),
                        };
                        Val::Ptr(Some(cell))
                    }
                    None => Val::Ptr(None),
                };
                set(st, inst, v);
                Ok(Flow::Next)
            }
            Operation::Load => {
                let p = inst.operands.first().copied().ok_or("load-arity")?;
                let cell = self.ptr(st, p)?;
                if self.goal == Goal::MemSafety {
                    let bytes = match inst.dst {
                        Some(d) if self.is_ptr.contains(&d) => {
                            u64::from(self.module.target_pointer_width)
                        }
                        _ => u64::from(self.result_bits(inst)?.div_ceil(8)),
                    };
                    self.ms_check_access(st, cell.as_ref(), bytes, inst.id, "load")?;
                }
                if cell.as_ref().is_some_and(|c| c.0 == NULL_BASE) {
                    trace(|| "load through NULL ends the path".to_string());
                    return Ok(Flow::Halt);
                }
                let d = inst.dst.ok_or("load-dst")?;
                let v = if self.is_ptr.contains(&d) {
                    self.load_pointer(st, cell.as_ref())
                } else {
                    let bits = self.result_bits(inst)?;
                    self.load_scalar(st, cell, bits)?
                };
                set(st, inst, v);
                Ok(Flow::Next)
            }
            Operation::Store => {
                let [v, p] = inst.operands[..] else {
                    return Err("store-arity".to_string());
                };
                if self.goal == Goal::MemSafety {
                    let cell = self.ptr(st, p)?;
                    let bytes = self.ms_value_bytes(v)?;
                    self.ms_check_access(st, cell.as_ref(), bytes, inst.id, "store")?;
                }
                let Some(cell) = self.ptr(st, p)? else {
                    trace(|| {
                        format!(
                            "store-unknown-pointer: pointer {p:?} is {}",
                            self.describe(p)
                        )
                    });
                    return Err("store-unknown-pointer".to_string());
                };
                if matches!(self.globals.get(&cell.0), Some((_, true))) {
                    return Err("store-const-global".to_string());
                }
                if cell.0 == NULL_BASE {
                    // Undefined behaviour: the program has no further behaviour
                    // on this path (the field's reading of a null dereference).
                    trace(|| "store through NULL ends the path".to_string());
                    return Ok(Flow::Halt);
                }
                if is_imprecise(&cell) {
                    // Some cell of the object changed: forget them all (later
                    // loads are fresh). The value itself need not be evaluated.
                    trace(|| format!("store at unknown offset havocs {:?}", cell.0));
                    st.memory.retain(|k, _| k.0 != cell.0);
                    st.escaped.insert(cell.0);
                    return Ok(Flow::Next);
                }
                let val = match self.val(st, v, self.width.get(&v).copied()) {
                    Ok(val) => val,
                    // Storing `undef`: the cell now holds any value. Its width is
                    // unknown here, so the object is marked escaped and the cell
                    // dropped (later loads are fresh) — more than needed, sound.
                    Err(e) if e == "undefined-value" => {
                        trace(|| format!("store of undef havocs {cell:?}"));
                        st.memory.remove(&cell);
                        st.escaped.insert(cell.0);
                        return Ok(Flow::Next);
                    }
                    Err(e) => return Err(e),
                };
                // The cell is exact from here on (cells are consulted before the
                // object's escape mark), whatever happened to the object before.
                self.write_cell(st, cell, val);
                Ok(Flow::Next)
            }
            // The block is left for good at its own terminator only: an inlined
            // call also jumps (into the callee) and comes back mid-block, so
            // pruning there dropped values still used after the call (lost the
            // XCSP proofs: `assume(var0 >= 0); assume(var0 <= 1);` in one block).
            Operation::Br { target } => {
                self.prune_block_locals(st);
                Ok(Flow::Jump(*target))
            }
            Operation::CondBr {
                then_target,
                else_target,
            } => {
                let c = inst.operands.first().copied().ok_or("condbr-arity")?;
                let (c, vars) = self.bv(st, c)?;
                if self.lazy() {
                    if let Some(split) = self.small_domain_split(st, &vars)? {
                        return Ok(Flow::Refine(split));
                    }
                }
                let t = truthy(&c);
                let f = t.clone().not();
                let mut targets = Vec::new();
                let then_ok = self.branch_feasible(st, &t, &vars)?;
                if then_ok {
                    targets.push((*then_target, t, vars.clone()));
                }
                // A state's path condition is satisfiable (or unknown), so when
                // the then-side is refuted the else-side holds without a query.
                if (!then_ok && !debug_flag("noelseskip")) || self.branch_feasible(st, &f, &vars)? {
                    targets.push((*else_target, f, vars));
                }
                self.prune_block_locals(st);
                Ok(Flow::Fork(targets))
            }
            Operation::Switch { default, cases } => {
                let v = inst.operands.first().copied().ok_or("switch-arity")?;
                let (v, vars) = self.bv(st, v)?;
                if self.lazy() {
                    if let Some(split) = self.small_domain_split(st, &vars)? {
                        return Ok(Flow::Refine(split));
                    }
                }
                let bits = v.get_size();
                let mut targets = Vec::new();
                let mut none_matched: Vec<Bool> = Vec::new();
                for (k, target) in cases {
                    let kb = int_bv(*k, bits);
                    let c = eq(&v, &kb);
                    none_matched.push(c.clone().not());
                    if self.branch_feasible(st, &c, &vars)? {
                        targets.push((*target, c, vars.clone()));
                    }
                }
                let refs: Vec<&Bool> = none_matched.iter().collect();
                let dflt = Bool::and(&refs);
                if self.branch_feasible(st, &dflt, &vars)? {
                    targets.push((*default, dflt, vars));
                }
                self.prune_block_locals(st);
                Ok(Flow::Fork(targets))
            }
            Operation::Ret => {
                let v = match inst.operands.first() {
                    Some(&r) => match self.val(st, r, self.width.get(&r).copied()) {
                        Ok(v) => Some(v),
                        // `return undef` (CIL leaves a function without a value):
                        // the caller gets any value of its result type (below).
                        Err(e) if e == "undefined-value" => None,
                        Err(e) => return Err(e),
                    },
                    None => None,
                };
                Ok(Flow::Return(v))
            }
            Operation::Unreachable => Ok(Flow::Halt),
            Operation::CallDirect { callee } => self.call(st, inst, *callee, &inst.operands),
            // The callee pointer is the LAST operand; it resolves when it is a
            // known function address (possibly stored and loaded through an
            // `ops` table in between).
            Operation::CallIndirect { .. } => {
                let Some((&callee_op, args)) = inst.operands.split_last() else {
                    return Err("call-indirect-arity".to_string());
                };
                match self.ptr(st, callee_op)? {
                    Some((base, path))
                        if path.is_empty()
                            && self
                                .module
                                .functions
                                .iter()
                                .any(|f| f.id.raw() == base.raw()) =>
                    {
                        trace(|| format!("indirect call resolved to {base:?}"));
                        self.call(st, inst, FunctionId::new(base.raw()), args)
                    }
                    _ => Err("call-indirect".to_string()),
                }
            }
            Operation::Global { .. } => Err("global-op".to_string()),
            // `memcpy`/`memset`: the destination object may now hold anything
            // (its cells are dropped and it is marked escaped: later loads are
            // fresh). The source is only read. An opaque destination could alias
            // any modelled cell — abstain, as for an unknown extern.
            Operation::Memcpy | Operation::Memset => {
                let dst = inst
                    .operands
                    .first()
                    .copied()
                    .ok_or("mem-intrinsic-arity")?;
                if self.goal == Goal::MemSafety {
                    // `(dst, src|val, len, ...)`: a concrete length, checked
                    // against the destination (and the source of a copy).
                    let len_id = inst.operands.get(2).copied().ok_or("mem-intrinsic-arity")?;
                    let len = match self.val(st, len_id, self.width.get(&len_id).copied())? {
                        Val::Bv(b, _) => b.simplify().as_u64(),
                        Val::Ptr(_) => None,
                    }
                    .ok_or("ms-mem-intrinsic-symbolic-length")?;
                    if len > 0 {
                        let cell = self.ptr(st, dst)?;
                        self.ms_check_access(st, cell.as_ref(), len, inst.id, "memwrite")?;
                        if matches!(inst.op, Operation::Memcpy) {
                            let src = inst.operands.get(1).copied().ok_or("mem-intrinsic-arity")?;
                            let cell = self.ptr(st, src)?;
                            self.ms_check_access(st, cell.as_ref(), len, inst.id, "memread")?;
                        }
                    }
                }
                let Some((base, _)) = self.ptr(st, dst)? else {
                    return Err("mem-intrinsic-opaque-dest".to_string());
                };
                if matches!(self.globals.get(&base), Some((_, true))) {
                    return Err("store-const-global".to_string());
                }
                trace(|| format!("mem-intrinsic havocs {base:?}"));
                st.memory.retain(|k, _| k.0 != base);
                st.escaped.insert(base);
                Ok(Flow::Next)
            }
        }
    }

    /// A pointer-typed load: the stored pointer; otherwise, for a cell never
    /// written on this path, NULL in a zero-allocated object, and the
    /// initialiser's value in a global (NULL, zero, or the address of another
    /// global / a function — how `static const struct ops = { .probe = &f }`
    /// resolves an indirect call). Opaque (`None`) for everything the engine
    /// cannot name, as before.
    /// The initialiser scalar starting exactly at `cell` (an exact offset into
    /// a modelled global with a layout) and its byte size.
    fn init_scalar(&self, cell: &Cell) -> Option<(Constant, u64)> {
        let off = cell_offset(cell)?;
        let (path, size) = self.layouts.get(&cell.0)?.scalar_at(u64::from(off))?;
        let (Some(init), _) = self.globals.get(&cell.0)? else {
            return None;
        };
        Some((init_at_path(init, &path)?, size))
    }

    /// Bytes a stored value occupies.
    fn val_bytes(&self, v: &Val) -> u32 {
        match v {
            Val::Bv(x, _) => x.get_size().div_ceil(8),
            Val::Ptr(_) => self.module.target_pointer_width,
        }
    }

    /// The written cells of `cell.0`, other than `cell` itself, that share a
    /// byte with the `bytes` bytes starting at `cell`. Empty for an unknown
    /// offset.
    fn overlapping_cells(&self, st: &State, cell: &Cell, bytes: u32) -> Vec<Cell> {
        // No modelled value is wider than 16 bytes, so an overlapping cell
        // starts fewer than 16 bytes before `off`.
        const MAX_CELL: u32 = 16;
        let Some(off) = cell_offset(cell) else {
            return Vec::new();
        };
        let lo = if off < MAX_CELL {
            (cell.0, Vec::new())
        } else {
            (cell.0, vec![off - MAX_CELL])
        };
        let end = u64::from(off) + u64::from(bytes);
        let hi_off =
            u32::try_from(end.saturating_sub(1)).map_or(IMPRECISE - 1, |o| o.min(IMPRECISE - 1));
        let hi = (cell.0, vec![hi_off.max(off)]);
        st.memory
            .range(lo..=hi)
            .filter(|(k, _)| *k != cell)
            .filter(|(k, v)| {
                cell_offset(k).is_some_and(|o| {
                    let w = u64::from(self.val_bytes(v));
                    u64::from(o) < end && u64::from(off) < u64::from(o) + w
                })
            })
            .map(|(k, _)| k.clone())
            .collect()
    }

    /// Store `v` at `cell`: cells it overlaps hold a partly overwritten value
    /// now (any value: they are dropped, so later loads of them are fresh).
    fn write_cell(&self, st: &mut State, cell: Cell, v: Val) {
        for k in self.overlapping_cells(st, &cell, self.val_bytes(&v)) {
            st.memory.remove(&k);
        }
        st.memory.insert(cell, v);
    }

    // ---- memory safety (Goal::MemSafety, plan 222 §2s) -----------------------

    /// Byte size of an alloca or global object; `None` when unknown.
    fn ms_static_size(&self, st: &State, base: ValueId) -> Option<u64> {
        let d = st.versioned.get(&base).copied().unwrap_or(base);
        if let Some(&n) = self.alloca_sizes.get(&d) {
            return Some(n);
        }
        if self.globals.contains_key(&base) {
            return self.layouts.get(&base).map(Layout::size);
        }
        None
    }

    /// `bytes` bytes at `cell` lie inside a live object; otherwise abstain
    /// (`error-feasible` for a feasible symbolic out-of-bounds access).
    fn ms_check_access(
        &mut self,
        st: &mut State,
        cell: Option<&Cell>,
        bytes: u64,
        at: InstId,
        what: &str,
    ) -> Res<()> {
        let Some(cell) = cell else {
            return Err(format!("ms-{what}-opaque-pointer"));
        };
        if cell.0 == NULL_BASE {
            return Err(format!("ms-{what}-null"));
        }
        let Some(off) = cell_offset(cell) else {
            return Err(format!("ms-{what}-unknown-offset"));
        };
        if st.dead.contains(&cell.0) {
            return Err(format!("ms-{what}-dangling-stack"));
        }
        let end = u64::from(off) + bytes;
        if let Some(h) = st.heap.get(&cell.0) {
            if h.status != HeapStatus::Live {
                return Err(format!("ms-{what}-not-live"));
            }
            let Some((size, vars)) = h.size.clone() else {
                return Err(format!("ms-{what}-unknown-size"));
            };
            if let Some(n) = size.simplify().as_u64() {
                return if end <= n {
                    Ok(())
                } else {
                    Err(format!("ms-{what}-out-of-bounds"))
                };
            }
            let end_bv = BV::from_u64(end, size.get_size());
            let oob = size.bvult(&end_bv);
            return self.check_violation(st, Some((&oob, &vars)), at);
        }
        let size = self
            .ms_static_size(st, cell.0)
            .ok_or_else(|| format!("ms-{what}-unknown-object"))?;
        if end <= size {
            Ok(())
        } else {
            trace(|| {
                format!(
                    "ms out of bounds at {at:?}: object {:?} ({}) offset {off} + {bytes} > size {size}",
                    cell.0,
                    self.describe(cell.0)
                )
            });
            Err(format!("ms-{what}-out-of-bounds"))
        }
    }

    /// Bytes a stored value occupies.
    fn ms_value_bytes(&self, v: ValueId) -> Res<u64> {
        if self.is_ptr.contains(&v)
            || matches!(
                self.module.constants.get(&v),
                Some(Constant::Null | Constant::GlobalRef(_) | Constant::GlobalElem { .. })
            )
        {
            return Ok(u64::from(self.module.target_pointer_width));
        }
        self.width
            .get(&v)
            .map(|b| u64::from(b.div_ceil(8)))
            .ok_or_else(|| "ms-store-width".to_string())
    }

    /// The byte size of a `malloc`/`calloc` block (64-bit), never-failing
    /// allocation (SV-COMP rules: `malloc` and `alloca` always succeed).
    fn ms_alloc_size(
        &mut self,
        st: &mut State,
        inst: &Instruction,
        kind: &saf_core::air::HeapAllocKind,
    ) -> Res<(BV, Vars)> {
        let arg = |this: &mut Self, st: &mut State, i: usize| -> Res<(BV, Vars)> {
            let id = inst.operands.get(i).copied().ok_or("ms-alloc-arity")?;
            let (b, vars) = this.bv(st, id)?;
            let b = match b.get_size() {
                64 => b,
                w if w < 64 => b.zero_ext(64 - w),
                _ => return Err("ms-alloc-size-width".to_string()),
            };
            Ok((b, vars))
        };
        match kind {
            saf_core::air::HeapAllocKind::Malloc => arg(self, st, 0),
            saf_core::air::HeapAllocKind::Calloc => {
                let (n, vn) = arg(self, st, 0)?;
                let (m, _) = arg(self, st, 1)?;
                let (Some(n), Some(m)) = (n.simplify().as_u64(), m.simplify().as_u64()) else {
                    return Err("ms-calloc-symbolic".to_string());
                };
                let size = n.checked_mul(m).ok_or("ms-calloc-overflow")?;
                Ok((BV::from_u64(size, 64), vn))
            }
            other => Err(format!("ms-allocator-{}", other.as_str())),
        }
    }

    /// A C object's lifetime marker (`llvm.lifetime.*`, renamed by the
    /// memory-safety ingestion recipe): `end` kills the stack object; `start`
    /// of an object that ended before begins a NEW object — a fresh one when any
    /// pointer to the old one survives anywhere (it must stay dead: `scopes3`,
    /// `derefInLoop1`), else the same one with indeterminate contents.
    fn ms_lifetime(&mut self, st: &mut State, args: &[ValueId], start: bool) -> Res<Flow> {
        let p = args.get(1).copied().ok_or("lifetime-arity")?;
        let Some((base, path)) = self.ptr(st, p)? else {
            return Err("ms-lifetime-opaque-pointer".to_string());
        };
        if !path.is_empty() {
            return Err("ms-lifetime-offset".to_string());
        }
        let d = st.versioned.get(&base).copied().unwrap_or(base);
        if !self.alloca_sizes.contains_key(&d) && !st.seen_alloca.contains(&d) {
            return Err("ms-lifetime-not-alloca".to_string());
        }
        if !start {
            st.dead.insert(base);
            return Ok(Flow::Next);
        }
        if !st.dead.contains(&base) {
            return Ok(Flow::Next); // the first start: already live
        }
        let referenced = st
            .memory
            .values()
            .chain(st.values.iter().filter(|(k, _)| **k != d).map(|(_, v)| v))
            .chain(
                st.frames
                    .iter()
                    .flat_map(|f| f.saved.iter().filter_map(|(_, v)| v.as_ref())),
            )
            .any(|v| matches!(v, Val::Ptr(Some((b, _))) if *b == base));
        let fresh = if referenced {
            self.alloca_versions += 1;
            let b = ValueId::new(u128::MAX / 2 - u128::from(self.alloca_versions));
            st.versioned.insert(b, d);
            st.values.insert(d, Val::Ptr(Some((b, Vec::new()))));
            st.stack.push((st.frames.len(), b));
            b
        } else {
            st.dead.remove(&base);
            base
        };
        st.memory.retain(|k, _| k.0 != fresh);
        st.escaped.remove(&fresh);
        Ok(Flow::Next)
    }

    /// `free(p)`: NULL is a no-op; the start of a live heap block frees it;
    /// anything else abstains.
    fn ms_free(&mut self, st: &mut State, args: &[ValueId]) -> Res<Flow> {
        let p = args.first().copied().ok_or("free-arity")?;
        match self.ptr(st, p)? {
            Some((b, _)) if b == NULL_BASE => Ok(Flow::Next),
            Some((b, path)) if path.is_empty() => match st.heap.get_mut(&b) {
                Some(h) if h.status == HeapStatus::Live => {
                    h.status = HeapStatus::Freed;
                    Ok(Flow::Next)
                }
                _ => Err("ms-invalid-free".to_string()),
            },
            _ => Err("ms-invalid-free".to_string()),
        }
    }

    /// `printf`-like externs read their pointer arguments as C strings: each
    /// must point into a constant string global (always NUL-terminated inside
    /// its object) at an offset inside it, and a format must not contain `%n`
    /// (a write). A `FILE *` stream argument is not read through.
    fn ms_pure_extern_args(&mut self, st: &mut State, name: &str, args: &[ValueId]) -> Res<()> {
        let stream = STREAM_ARG.iter().find(|(n, _)| *n == name).map(|(_, i)| *i);
        let format_at = match name {
            "printf" => Some(0),
            "fprintf" => Some(1),
            _ => None,
        };
        for (i, &arg) in args.iter().enumerate() {
            if stream == Some(i)
                || !matches!(
                    self.val(st, arg, self.width.get(&arg).copied())?,
                    Val::Ptr(_)
                )
            {
                continue;
            }
            let text = match self.ptr(st, arg)? {
                Some(cell) => match (cell_offset(&cell), self.globals.get(&cell.0)) {
                    (Some(off), Some((Some(Constant::String { value }), true)))
                        if self
                            .layouts
                            .get(&cell.0)
                            .is_some_and(|l| u64::from(off) < l.size()) =>
                    {
                        Some(
                            value
                                .get(usize::try_from(off).unwrap_or(usize::MAX)..)
                                .unwrap_or(""),
                        )
                    }
                    _ => None,
                },
                None => None,
            };
            let Some(text) = text else {
                return Err(format!("ms-extern-{name}-pointer-arg"));
            };
            if format_at == Some(i) && format_writes(text) {
                return Err(format!("ms-extern-{name}-percent-n"));
            }
        }
        Ok(())
    }

    /// At a path end every unfreed heap block must be reachable from the
    /// globals. A block that became untracked earlier can never become tracked
    /// again, so this also catches a leak anywhere on the (finite) path; the
    /// call stack is not a root here, which is stricter than the definition.
    fn ms_memtrack(&self, st: &State) -> Res<()> {
        let live: Vec<ValueId> = st
            .heap
            .iter()
            .filter(|(_, h)| h.status != HeapStatus::Freed)
            .map(|(b, _)| *b)
            .collect();
        if live.is_empty() {
            return Ok(());
        }
        let mut reached: BTreeSet<ValueId> = BTreeSet::new();
        let mut work: Vec<ValueId> = self.globals.keys().copied().collect();
        let mut unknown = false;
        while let Some(b) = work.pop() {
            if !reached.insert(b) {
                continue;
            }
            if st.escaped.contains(&b) {
                unknown = true;
            }
            for (k, v) in st.memory.range((b, Vec::new())..) {
                if k.0 != b {
                    break;
                }
                match v {
                    Val::Ptr(Some((t, _))) => work.push(*t),
                    Val::Ptr(None) => unknown = true,
                    Val::Bv(..) => {}
                }
            }
        }
        if live.iter().all(|b| reached.contains(b)) {
            Ok(())
        } else if unknown {
            Err("ms-memtrack-unknown".to_string())
        } else {
            Err("ms-leak".to_string())
        }
    }

    fn load_pointer(&mut self, st: &State, cell: Option<&Cell>) -> Val {
        let Some(cell) = cell else {
            return Val::Ptr(None);
        };
        if let Some(v) = st.memory.get(cell) {
            return match v {
                Val::Ptr(q) => Val::Ptr(q.clone()),
                Val::Bv(..) => Val::Ptr(None),
            };
        }
        if is_imprecise(cell) || st.escaped.contains(&cell.0) {
            return Val::Ptr(None);
        }
        if !self
            .overlapping_cells(st, cell, self.module.target_pointer_width)
            .is_empty()
        {
            return Val::Ptr(None);
        }
        if st.zeroed.contains(&cell.0) {
            return Val::Ptr(Some((NULL_BASE, Vec::new())));
        }
        if self.globals.contains_key(&cell.0) {
            return match self.init_scalar(cell) {
                Some((Constant::Null | Constant::ZeroInit, _)) => {
                    Val::Ptr(Some((NULL_BASE, Vec::new())))
                }
                Some((c @ (Constant::GlobalRef(_) | Constant::GlobalElem { .. }), _)) => {
                    match self.constant(&c, None) {
                        Ok(v @ Val::Ptr(_)) => v,
                        _ => Val::Ptr(None),
                    }
                }
                _ => Val::Ptr(None),
            };
        }
        Val::Ptr(None)
    }

    fn load_scalar(&mut self, st: &mut State, cell: Option<Cell>, bits: u32) -> Res<Val> {
        let Some(cell) = cell else {
            trace(|| "fresh: load through opaque pointer".to_string());
            return Ok(self.fresh_val(bits));
        };
        // A written cell is exact, even inside an escaped object: nothing the
        // engine models can write a cell behind its back (opaque stores abstain).
        if let Some(v) = st.memory.get(&cell) {
            return match v {
                Val::Bv(x, vs) if x.get_size() == bits => Ok(Val::Bv(x.clone(), vs.clone())),
                Val::Bv(..) => Err("load-width-mismatch".to_string()),
                Val::Ptr(_) => Err("load-int-from-pointer-cell".to_string()),
            };
        }
        if is_imprecise(&cell) {
            trace(|| format!("fresh: load at unknown offset in {:?}", cell.0));
            return Ok(self.fresh_val(bits));
        }
        // Part of the loaded bytes was written through another cell (a
        // narrower or misaligned store): the value is partly unknown.
        if !self
            .overlapping_cells(st, &cell, bits.div_ceil(8))
            .is_empty()
        {
            trace(|| format!("fresh: load overlaps a written cell of {:?}", cell.0));
            return Ok(self.fresh_val(bits));
        }
        if st.escaped.contains(&cell.0) {
            trace(|| {
                format!(
                    "fresh: load from escaped object {:?} path {:?}",
                    cell.0, cell.1
                )
            });
            // Whoever wrote the object wrote SOME value: the cell holds it from
            // here until the next store (or havoc), so two reads agree — a guard
            // `if (data < MAX)` and the `data++` it protects read the same value.
            // Reading a different fresh symbol each time lost every Juliet
            // `fscanf`/`recv` task to a spurious overflow (2026-10-10).
            let v = self.fresh_val(bits);
            st.memory.insert(cell, v.clone());
            return Ok(v);
        }
        // Never written on this path: zero for a zero-allocated heap object,
        // the global's initialiser at that path, an extern or uninitialised
        // object otherwise (any value).
        if st.zeroed.contains(&cell.0) {
            return Ok(Val::Bv(int_bv(0, bits), no_vars()));
        }
        if self.globals.contains_key(&cell.0) {
            let Some((c, size)) = self.init_scalar(&cell) else {
                trace(|| format!("fresh: initialiser has no scalar at {cell:?}"));
                return Ok(self.fresh_val(bits));
            };
            if size.saturating_mul(8) != u64::from(bits) {
                // A narrower or wider view of an initialised scalar: any value.
                trace(|| {
                    format!(
                        "fresh: load width {bits} differs from the initialised scalar at {cell:?}"
                    )
                });
                return Ok(self.fresh_val(bits));
            }
            return match self.constant(&c, Some(bits))? {
                v @ Val::Bv(..) if matches!(&v, Val::Bv(x, _) if x.get_size() == bits) => Ok(v),
                Val::Bv(..) => Err("global-init-width".to_string()),
                Val::Ptr(_) => Err("load-int-from-pointer-cell".to_string()),
            };
        }
        trace(|| {
            format!(
                "fresh: never-written cell {:?} path {:?} (no initialiser)",
                cell.0, cell.1
            )
        });
        Ok(self.fresh_val(bits))
    }

    fn binop(&mut self, st: &mut State, kind: BinaryOp, a: &BV, b: &BV, vars: &Vars) -> Res<BV> {
        let bits = a.get_size();
        let bool_bv = |c: Bool| c.ite(&int_bv(1, 1), &int_bv(0, 1));
        Ok(match kind {
            BinaryOp::Add => a.bvadd(b),
            BinaryOp::Sub => a.bvsub(b),
            BinaryOp::Mul => a.bvmul(b),
            BinaryOp::SDiv | BinaryOp::UDiv | BinaryOp::SRem | BinaryOp::URem => {
                // Division by zero is UB: only the `b != 0` continuation exists.
                let nz = eq(b, &int_bv(0, bits)).not();
                if !self.branch_feasible(st, &nz, vars)? {
                    return Err("division-by-zero".to_string());
                }
                st.constraints.push((nz, vars.clone()));
                match kind {
                    BinaryOp::SDiv => a.bvsdiv(b),
                    BinaryOp::UDiv => a.bvudiv(b),
                    BinaryOp::SRem => a.bvsrem(b),
                    _ => a.bvurem(b),
                }
            }
            BinaryOp::And => a.bvand(b),
            BinaryOp::Or => a.bvor(b),
            BinaryOp::Xor => a.bvxor(b),
            BinaryOp::Shl => a.bvshl(b),
            BinaryOp::LShr => a.bvlshr(b),
            BinaryOp::AShr => a.bvashr(b),
            BinaryOp::ICmpEq => bool_bv(eq(a, b)),
            BinaryOp::ICmpNe => bool_bv(eq(a, b).not()),
            BinaryOp::ICmpSlt => bool_bv(a.bvslt(b)),
            BinaryOp::ICmpSle => bool_bv(a.bvsle(b)),
            BinaryOp::ICmpSgt => bool_bv(a.bvsgt(b)),
            BinaryOp::ICmpSge => bool_bv(a.bvsge(b)),
            BinaryOp::ICmpUlt => bool_bv(a.bvult(b)),
            BinaryOp::ICmpUle => bool_bv(a.bvule(b)),
            BinaryOp::ICmpUgt => bool_bv(a.bvugt(b)),
            BinaryOp::ICmpUge => bool_bv(a.bvuge(b)),
            BinaryOp::FAdd | BinaryOp::FSub | BinaryOp::FMul | BinaryOp::FDiv => {
                fpa::arith(kind, a, b)?
            }
            BinaryOp::FCmpOeq
            | BinaryOp::FCmpOne
            | BinaryOp::FCmpOgt
            | BinaryOp::FCmpOge
            | BinaryOp::FCmpOlt
            | BinaryOp::FCmpOle
            | BinaryOp::FCmpUeq
            | BinaryOp::FCmpUne
            | BinaryOp::FCmpUgt
            | BinaryOp::FCmpUge
            | BinaryOp::FCmpUlt
            | BinaryOp::FCmpUle
            | BinaryOp::FCmpOrd
            | BinaryOp::FCmpUno
            | BinaryOp::FCmpTrue
            | BinaryOp::FCmpFalse => bool_bv(fpa::cmp(kind, a, b)?),
            BinaryOp::FRem => return Err(format!("binop-{kind:?}")),
        })
    }

    // NOTE: one dispatch over every modelled call kind (error, terminating,
    // assume, nondet, pure, unknown extern, inlined); splitting it would scatter
    // the allow-list order the soundness argument depends on.
    #[allow(clippy::too_many_lines)]
    fn call(
        &mut self,
        st: &mut State,
        inst: &Instruction,
        callee: FunctionId,
        args: &[ValueId],
    ) -> Res<Flow> {
        let f = self.module.function(callee).ok_or("callee-missing")?;
        let name = f.name.as_str();
        if REACH_ERROR_NAMES.contains(&name) {
            trace(|| {
                format!(
                    "error call: mode={:?} visits={:?} frames={} constraints={}",
                    self.mode,
                    st.visits,
                    st.frames.len(),
                    st.constraints.len()
                )
            });
            // Under `no-overflow` the call is an abort like any other; the
            // violation is decided at the arithmetic, not here.
            if self.goal == Goal::ReachError {
                self.check_violation(st, None, inst.id)?;
            }
            return Ok(Flow::Halt);
        }
        // A function the task DEFINES is executed, whatever its name: the
        // name-based models below are for the externs only. (`ldv-consumption`
        // defines `ldv_malloc`/`kfree` with counting bodies; a task may define
        // its own `free`, `abs` or `sqrt`.) `reach_error` and the assumption
        // intrinsics keep their meaning either way.
        let defined = !f.is_declaration && !f.blocks.is_empty();
        if !defined && TERMINATING.contains(&name) {
            return Ok(Flow::Halt);
        }
        if !defined && self.goal == Goal::MemSafety && name == "free" {
            return self.ms_free(st, args);
        }
        if !defined && name == KEEP_ALIVE {
            return Ok(Flow::Next);
        }
        if !defined && (name == LIFETIME_START || name == LIFETIME_END) {
            if self.goal != Goal::MemSafety {
                return Ok(Flow::Next);
            }
            return self.ms_lifetime(st, args, name == LIFETIME_START);
        }
        if !defined
            && self.goal == Goal::MemSafety
            && NOOP_EXTERNS.contains(&name)
            && name != "srand"
        {
            return Err(format!("ms-deallocator-{name}"));
        }
        if !defined && NOOP_EXTERNS.contains(&name) {
            return Ok(Flow::Next);
        }
        if ASSUME.contains(&name) {
            let c = args.first().copied().ok_or("assume-arity")?;
            let (c, vars) = self.bv(st, c)?;
            let t = truthy(&c);
            if !self.branch_feasible(st, &t, &vars)? {
                st.dead_end = true;
                return Ok(Flow::Halt);
            }
            st.constraints.push((t, vars));
            return Ok(Flow::Next);
        }
        // `rand` is a pure source of values too (its state is libc's, not the
        // program's), so it is a nondet read rather than a havocking extern.
        let float_nondet = matches!(name, "__VERIFIER_nondet_float" | "__VERIFIER_nondet_double");
        if !defined && (is_scalar_integer_nondet(name) || name == "rand" || float_nondet) {
            let bits = self.result_bits(inst)?;
            let v = self.fresh_val(bits);
            // A float read is not recorded: the replay driver cannot hand a
            // bit pattern back through `__VERIFIER_nondet_double`.
            if self.cex_mode && !float_nondet {
                st.nondets
                    .push((name.to_string(), bits, self.fresh_counter - 1, inst.id));
            }
            if let Some(d) = inst.dst {
                st.values.insert(d, v);
            }
            return Ok(Flow::Next);
        }
        if name.starts_with("llvm.dbg.") || name.starts_with("llvm.lifetime.") {
            return Ok(Flow::Next);
        }
        if !defined {
            if let Some(flow) = self.pure_math(st, inst, name, args)? {
                return Ok(flow);
            }
        }
        if !defined && PURE_EXTERNS.contains(&name) {
            if self.goal == Goal::MemSafety {
                self.ms_pure_extern_args(st, name, args)?;
            }
            if let Some(d) = inst.dst {
                let v = match self.width.get(&d) {
                    Some(&w) => self.fresh_val(w),
                    None => Val::Ptr(None),
                };
                st.values.insert(d, v);
            }
            return Ok(Flow::Next);
        }
        if !defined && NON_LOCAL_CONTROL.contains(&name) {
            return Err(format!("extern-{name}-nonlocal-control"));
        }
        if f.is_declaration || f.blocks.is_empty() {
            return self.call_unknown_extern(st, inst, name, args);
        }
        if st.frames.len() >= MAX_CALL_DEPTH {
            return Err("call-depth".to_string());
        }
        // Recursion is admitted when the callee has no `alloca` (an inner
        // activation's cell would alias the outer's); the outer activation's
        // SSA bindings are saved in the frame and restored on return. Depth is
        // bounded by the frame cap, which abstains (`call-depth`) — a recursion
        // bounded by concrete arguments unwinds to completion like a loop.
        let recursive = st.func == callee || st.frames.iter().any(|fr| fr.func == callee);
        if recursive && self.has_alloca.contains(&callee) {
            return Err("recursion-with-alloca".to_string());
        }
        // Bind arguments, then transfer control to the callee's entry block.
        // Extra arguments are ignored (a defined variadic function reads them
        // only through `va_start`, and a module with that intrinsic is gated
        // out before the engine runs); a missing one (K&R-style prototype
        // mismatch) is any value of the parameter's type.
        let mut bound = Vec::with_capacity(f.params.len());
        for (i, p) in f.params.iter().enumerate() {
            let Some(&arg) = args.get(i) else {
                let v = match self.width.get(&p.id) {
                    Some(&w) => self.fresh_val(w),
                    None if self.is_ptr.contains(&p.id) => Val::Ptr(None),
                    None => return Err("call-arity".to_string()),
                };
                bound.push((p.id, v));
                continue;
            };
            let v = match self.val(st, arg, self.width.get(&arg).copied()) {
                Ok(v) => v,
                // An argument the module defines nowhere is an `undef` the
                // frontend did not record (CIL's uninitialised `var_groupN`
                // locals in every LDV driver harness): any value of the
                // parameter's type.
                Err(e) if e == "undefined-value" => match self.width.get(&p.id) {
                    Some(&w) => self.fresh_val(w),
                    None if self.is_ptr.contains(&p.id) => Val::Ptr(None),
                    None => {
                        trace(|| format!("arg {arg:?} of {name}: {e} ({})", self.describe(arg)));
                        return Err(e);
                    }
                },
                Err(e) => return Err(e),
            };
            match (&v, self.width.get(&p.id)) {
                (Val::Bv(b, _), Some(&w)) if b.get_size() == w => {}
                (Val::Ptr(_), None) if self.is_ptr.contains(&p.id) => {}
                _ => return Err("call-arg-type".to_string()),
            }
            bound.push((p.id, v));
        }
        let entry = f
            .entry_block
            .or_else(|| f.blocks.first().map(|b| b.id))
            .ok_or("callee-entry")?;
        let saved = if recursive {
            self.owned
                .get(&callee)
                .map(|vs| vs.iter().map(|v| (*v, st.values.get(v).cloned())).collect())
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        st.frames.push(Frame {
            func: st.func,
            ret_block: st.block,
            ret_idx: st.idx,
            ret_prev: st.prev,
            ret_dst: inst.dst,
            saved,
        });
        for (pid, v) in bound {
            st.values.insert(pid, v);
        }
        st.func = callee;
        st.prev = None;
        Ok(Flow::Jump(entry))
    }

    /// A call to a function with no body that is on no allow-list. Modelled by
    /// the only sound over-approximation available without its code: it may
    /// return ANY value, and it may have written ANY value to every global and to
    /// every modelled cell whose address it was given — so the result is fresh,
    /// every global cell and every pointee cell is havocked (marked escaped: a
    /// later load is fresh, never the initialiser), and the path continues.
    /// Assuming it returns only ADDS behaviours (a non-returning extern ends the
    /// path earlier). Two things it cannot be allowed to do: receive an OPAQUE
    /// pointer (its target could alias a modelled cell we would not havoc), or
    /// run while some modelled cell holds a pointer (it could follow it to a
    /// program function and reach the error through a callback). Both abstain.
    /// The pure math builtins the engine evaluates exactly — libm functions and
    /// the LLVM intrinsics clang emits for them (`abs()` is `llvm.abs.i32` even
    /// at `-O0`). `Ok(None)`: not a builtin. Integer `abs` of the minimum value
    /// is a signed overflow under the `no-overflow` goal (`imaxabs(INT64_MIN)`
    /// is what Juliet's `*_bad_abs` tasks expect FALSE for); `fmuladd` may be
    /// fused or not, so its result is either, chosen freshly.
    // NOTE: `x`, `y`, `z`, `r`, `d` are the operands, result and destination
    // of a one-line math builtin; longer names would not read better.
    #[allow(clippy::many_single_char_names)]
    fn pure_math(
        &mut self,
        st: &mut State,
        inst: &Instruction,
        name: &str,
        args: &[ValueId],
    ) -> Res<Option<Flow>> {
        let base = name
            .strip_prefix("llvm.")
            .map_or(name, |rest| rest.split('.').next().unwrap_or(rest));
        let family: &str = match (name.starts_with("llvm."), base) {
            (true, "abs") | (false, "abs" | "labs" | "llabs" | "imaxabs") => "iabs",
            (true, "smax" | "smin" | "umax" | "umin") => base,
            (true, "fabs") | (false, "fabs" | "fabsf" | "fabsl") => "fabs",
            (true, "sqrt") | (false, "sqrt" | "sqrtf" | "sqrtl") => "sqrt",
            (true, "floor") | (false, "floor" | "floorf" | "floorl") => "floor",
            (true, "ceil") | (false, "ceil" | "ceilf" | "ceill") => "ceil",
            (true, "trunc") | (false, "trunc" | "truncf" | "truncl") => "trunc",
            (true, "round") | (false, "round" | "roundf" | "roundl") => "round",
            (true, "rint" | "nearbyint")
            | (false, "rint" | "rintf" | "rintl" | "nearbyint" | "nearbyintf" | "nearbyintl") => {
                "rint"
            }
            (true, "minnum") | (false, "fmin" | "fminf" | "fminl") => "fmin",
            (true, "maxnum") | (false, "fmax" | "fmaxf" | "fmaxl") => "fmax",
            (true, "fmuladd" | "fma") | (false, "fma" | "fmaf" | "fmal") => "fma",
            (true, "copysign") | (false, "copysign" | "copysignf" | "copysignl") => "copysign",
            _ => return Ok(None),
        };
        let Some(d) = inst.dst else {
            return Ok(None);
        };
        let bits = self.result_bits(inst)?;
        let arg = |this: &mut Self, st: &mut State, i: usize| -> Res<(BV, Vars)> {
            let id = *args.get(i).ok_or("builtin-arity")?;
            let (v, vars) = this.bv(st, id)?;
            if v.get_size() != bits {
                return Err("builtin-width".to_string());
            }
            Ok((v, vars))
        };
        let (r, vars) = match family {
            "iabs" => {
                let (x, vars) = arg(self, st, 0)?;
                if self.goal == Goal::SignedOverflow {
                    let min = int_bv(1, bits).bvshl(int_bv(i64::from(bits) - 1, bits));
                    self.check_violation(st, Some((&eq(&x, &min), &vars)), inst.id)?;
                }
                let neg = int_bv(0, bits).bvsub(&x);
                (x.bvslt(int_bv(0, bits)).ite(&neg, &x), vars)
            }
            "smax" | "smin" | "umax" | "umin" => {
                let (x, vx) = arg(self, st, 0)?;
                let (y, vy) = arg(self, st, 1)?;
                let pick_x = match family {
                    "smax" => x.bvsgt(&y),
                    "smin" => x.bvslt(&y),
                    "umax" => x.bvugt(&y),
                    _ => x.bvult(&y),
                };
                (pick_x.ite(&x, &y), union(&vx, &vy))
            }
            "fabs" | "sqrt" | "floor" | "ceil" | "trunc" | "round" | "rint" => {
                let (x, vars) = arg(self, st, 0)?;
                (fpa::unary(family, &x)?, vars)
            }
            "fmin" | "fmax" => {
                let (x, vx) = arg(self, st, 0)?;
                let (y, vy) = arg(self, st, 1)?;
                (fpa::minmax(family == "fmin", &x, &y)?, union(&vx, &vy))
            }
            "fma" => {
                let (x, vx) = arg(self, st, 0)?;
                let (y, vy) = arg(self, st, 1)?;
                let (z, vz) = arg(self, st, 2)?;
                let fused = fpa::fma(&x, &y, &z)?;
                let r = if name.starts_with("llvm.fmuladd") {
                    // Fused or not is the implementation's choice.
                    let unfused =
                        fpa::arith(BinaryOp::FAdd, &fpa::arith(BinaryOp::FMul, &x, &y)?, &z)?;
                    let (pick, pv) = self.fresh(1);
                    let r = eq(&pick, &int_bv(1, 1)).ite(&fused, &unfused);
                    return Ok(Some(Self::finish_builtin(
                        st,
                        d,
                        r,
                        union(&union(&union(&vx, &vy), &vz), &pv),
                    )));
                } else {
                    fused
                };
                (r, union(&union(&vx, &vy), &vz))
            }
            "copysign" => {
                let (x, vx) = arg(self, st, 0)?;
                let (y, vy) = arg(self, st, 1)?;
                let sign = int_bv(1, bits).bvshl(int_bv(i64::from(bits) - 1, bits));
                let r = x.bvand(sign.bvnot()).bvor(y.bvand(&sign));
                (r, union(&vx, &vy))
            }
            _ => return Ok(None),
        };
        Ok(Some(Self::finish_builtin(st, d, r, vars)))
    }

    fn finish_builtin(st: &mut State, d: ValueId, r: BV, vars: Vars) -> Flow {
        st.values.insert(d, Self::mk(r, vars));
        Flow::Next
    }

    /// A `shl` whose source location is known NOT to be a signed shift (clang's
    /// `-fsanitize=shift-base` emitted no check there): an unsigned shift is
    /// defined for every operand. Without the location table, or without a
    /// location on the instruction, the shift counts as signed (checked).
    fn shl_is_unsigned(&self, inst: &Instruction) -> bool {
        let (Some(set), Some(span)) = (&self.signed_shl_locs, inst.span.as_ref()) else {
            return false;
        };
        crate::witness_lower::span_to_location(self.module, span)
            .is_some_and(|(_, line, col)| !set.contains(&(line, col)))
    }

    fn call_unknown_extern(
        &mut self,
        st: &mut State,
        inst: &Instruction,
        name: &str,
        args: &[ValueId],
    ) -> Res<Flow> {
        if self.goal == Goal::MemSafety {
            let stream = STREAM_ARG.iter().find(|(n, _)| *n == name).map(|(_, i)| *i);
            for (i, &arg) in args.iter().enumerate() {
                if stream == Some(i)
                    || matches!(self.module.constants.get(&arg), Some(Constant::Null))
                {
                    continue;
                }
                if matches!(
                    self.val(st, arg, self.width.get(&arg).copied())?,
                    Val::Ptr(_)
                ) {
                    return Err(format!("ms-extern-{name}-pointer-arg"));
                }
            }
        }
        let mut pointee: Vec<ValueId> = Vec::new();
        let stream_arg = STREAM_ARG.iter().find(|(n, _)| *n == name).map(|(_, i)| *i);
        // Counterexample mode models `fgets` as returning its buffer (the
        // driver's wrapper does the same); the buffer's contents are havocked
        // below like any pointee.
        let fgets_ret = match args.first() {
            Some(&buf) if self.cex_mode && name == "fgets" => Some(self.val(st, buf, None)?),
            _ => None,
        };
        for (i, &arg) in args.iter().enumerate() {
            // A C stream (`FILE *`, e.g. the loaded `stdin`) is a libc object no
            // program cell can alias: harmless though opaque.
            if stream_arg == Some(i) {
                continue;
            }
            // A null-pointer constant (`time(NULL)`, `srand(time(0))` in every
            // Juliet task) points at nothing the callee could write.
            if matches!(self.module.constants.get(&arg), Some(Constant::Null)) {
                continue;
            }
            let v = self.val(st, arg, self.width.get(&arg).copied())?;
            match v {
                Val::Ptr(Some((base, _))) => pointee.push(base),
                // The counterexample search is replay-gated: it may assume
                // the callee writes nothing through a pointer it cannot name
                // (the prover must not, and abstains).
                Val::Ptr(None) if self.cex_mode => {}
                Val::Ptr(None) => return Err(format!("extern-{name}-opaque-pointer-arg")),
                Val::Bv(..) => {}
            }
        }
        if !self.cex_mode && st.memory.values().any(|v| matches!(v, Val::Ptr(_))) {
            return Err(format!("extern-{name}-pointer-in-memory"));
        }
        trace(|| format!("fresh: unknown extern {name} havocs globals + pointee args"));
        let globals: Vec<ValueId> = self.globals.keys().copied().collect();
        for base in globals.into_iter().chain(pointee) {
            st.memory.retain(|k, _| k.0 != base);
            st.escaped.insert(base);
        }
        if self.cex_mode && (name == "fscanf" || name == "__isoc99_fscanf") {
            self.pin_fscanf_writes(st, inst, name, args)?;
        }
        if let (Some(d), Some(v)) = (inst.dst, fgets_ret) {
            st.values.insert(d, v);
            // Recorded as a read pinned to 1 ("returned the buffer") so the
            // replay driver knows to model it the same way.
            let (_, _) = self.fresh(32);
            let n = self.fresh_counter - 1;
            st.pinned.insert(n, 1);
            st.nondets.push((name.to_string(), 32, n, inst.id));
            return Ok(Flow::Next);
        }
        if let Some(d) = inst.dst {
            match self.width.get(&d) {
                Some(&w) => {
                    let v = self.fresh_val(w);
                    // Counterexample mode records EVERY unknown extern's
                    // integer result: the replay driver wraps the
                    // `PINNED_EXTERNS` and stubs the functions the task
                    // declares but never defines (LDV `ldv_undef_int`, …) with
                    // the model's values. Only the pinned externs get witness
                    // waypoints (a validator knows libc's semantics).
                    if self.cex_mode {
                        let n = self.fresh_counter - 1;
                        st.nondets.push((name.to_string(), w, n, inst.id));
                        if SMALL_RESULT_EXTERNS.contains(&name) {
                            if let Val::Bv(sym, vars) = &v {
                                let lo = sym.bvsge(int_bv(0, w));
                                let hi = sym.bvsle(int_bv(8, w));
                                st.constraints.push((Bool::and(&[&lo, &hi]), vars.clone()));
                            }
                        }
                    }
                    st.values.insert(d, v);
                }
                None if self.is_ptr.contains(&d) => {
                    st.values.insert(d, Val::Ptr(None));
                }
                // No width and not a pointer: a `void` call (`srand`) or a
                // result of a type the engine does not model. Left undefined:
                // a later read gets a fresh value if its width is known there,
                // or abstains (`undefined-value`) — never a wrong value.
                None => trace(|| format!("extern {name}: result left undefined")),
            }
        }
        Ok(Flow::Next)
    }

    // ---- solver ----------------------------------------------------------

    /// A potential violation at the current point: a `reach_error` call
    /// (`cond == None`), or signed arithmetic that overflows exactly when `cond`
    /// holds. `Ok(())` means no feasible violation here (the path goes on, or
    /// ends, as the caller sees fit); `Err` is the engine's verdict on it.
    ///
    /// Step case of k-induction, inside the first k iterations: the inductive
    /// hypothesis says this path is violation-free, so nothing is checked.
    /// Otherwise the path condition — extended by `cond` — is decided: eagerly
    /// branching modes have already found it satisfiable (unless a query came
    /// back UNKNOWN), lazy ones decide it here as one query over the whole path.
    fn check_violation(
        &mut self,
        st: &mut State,
        cond: Option<(&Bool, &Vars)>,
        at: InstId,
    ) -> Res<()> {
        if self.cex_mode && self.cex_skip.contains(&at) {
            return Ok(()); // a candidate here did not replay: look further
        }
        if let (Mode::Step { k }, Some(lp)) = (self.mode, self.kloop.as_ref()) {
            // Iteration count is kept per path, so a violation reached from a
            // nested call still belongs to the current iteration.
            if st.visits.get(&lp.header).copied().unwrap_or(0) <= k {
                return Ok(());
            }
        }
        self.violation_query = true;
        let feas = match cond {
            None => {
                if self.lazy() && !self.path_alive(st)? {
                    self.violation_query = false;
                    return Ok(()); // a dead path
                }
                Ok(Feas::Sat)
            }
            Some((c, _)) if self.lazy() => {
                let (pc, _) = path_condition(st);
                let query = Bool::and(&[&pc, c]);
                self.feasible(st, &query, &no_vars())
            }
            Some((c, vars)) => self.feasible(st, c, vars),
        };
        self.violation_query = false;
        let feas = feas?;
        if self.cex_mode && feas == Feas::Sat && !st.has_unknown {
            self.record_counterexample(st, cond.map(|(c, _)| c), at)?;
        }
        match feas {
            Feas::Unsat => Ok(()),
            Feas::Unknown => Err("error-unknown".to_string()),
            Feas::Sat if st.has_unknown => Err("error-unknown".to_string()),
            Feas::Sat => Err("error-feasible".to_string()),
        }
    }

    /// Counterexample mode, `fscanf(stream, fmt, &a, &b, …)`: each integer
    /// conversion of the (constant) format writes ONE fresh value into its
    /// pointee cell right here, and that value is recorded as a pinned read, so
    /// the replay driver's `fscanf` wrapper can hand the model's values back in
    /// the same order. Widths follow the conversion (`%d` 32, `%hd` 16, `%hhd` /
    /// `%c` 8, `%lld` 64, `%ld`/`%zu` the data model's long). A format that is
    /// not a constant or has an unsupported conversion pins nothing (the havoc
    /// above already made the cells fresh; the replay then simply may not
    /// reproduce).
    fn pin_fscanf_writes(
        &mut self,
        st: &mut State,
        inst: &Instruction,
        name: &str,
        args: &[ValueId],
    ) -> Res<()> {
        let Some(&fmt_id) = args.get(1) else {
            return Ok(());
        };
        let fmt = match self.module.constants.get(&fmt_id) {
            Some(Constant::String { value }) => value.clone(),
            _ => match self.module.globals.iter().find(|g| g.id == fmt_id) {
                Some(g) => match &g.init {
                    Some(Constant::String { value }) => value.clone(),
                    _ => return Ok(()),
                },
                None => return Ok(()),
            },
        };
        let long_bits = self.module.target_pointer_width * 8;
        let mut convs: Vec<u32> = Vec::new();
        let bytes = fmt.as_bytes();
        let mut pos = 0;
        while pos < bytes.len() {
            if bytes[pos] != b'%' {
                pos += 1;
                continue;
            }
            pos += 1;
            if pos < bytes.len() && bytes[pos] == b'%' {
                pos += 1;
                continue;
            }
            let (mut shorts, mut longs, mut size_t_mod) = (0, 0, false);
            while pos < bytes.len() && matches!(bytes[pos], b'h' | b'l' | b'z') {
                match bytes[pos] {
                    b'h' => shorts += 1,
                    b'l' => longs += 1,
                    _ => size_t_mod = true,
                }
                pos += 1;
            }
            let Some(&conv) = bytes.get(pos) else {
                return Ok(());
            };
            pos += 1;
            let bits = match conv {
                b'c' => 8,
                b'd' | b'i' | b'u' | b'x' => {
                    if size_t_mod {
                        long_bits
                    } else if shorts >= 2 {
                        8
                    } else if shorts == 1 {
                        16
                    } else if longs == 0 {
                        32
                    } else if longs == 1 {
                        long_bits
                    } else {
                        64
                    }
                }
                _ => return Ok(()), // `%s`, `%f`, …: not pinnable
            };
            convs.push(bits);
        }
        for (k, bits) in convs.into_iter().enumerate() {
            let Some(&ptr_id) = args.get(2 + k) else {
                return Ok(());
            };
            let Some(cell) = self.ptr(st, ptr_id)? else {
                return Ok(());
            };
            if is_imprecise(&cell) || cell.0 == NULL_BASE {
                return Ok(());
            }
            let (sym, vars) = self.fresh(bits);
            let n = self.fresh_counter - 1;
            self.write_cell(st, cell, Val::Bv(sym, vars));
            st.nondets.push((name.to_string(), bits, n, inst.id));
        }
        Ok(())
    }

    /// Counterexample mode: one query over the WHOLE path condition (plus the
    /// violation condition) yields a model; every nondet read of the path takes
    /// its model value (completion gives the unconstrained ones any value).
    fn record_counterexample(
        &mut self,
        st: &mut State,
        cond: Option<&Bool>,
        at: InstId,
    ) -> Res<()> {
        let (pc, _) = path_condition(st);
        let query = match cond {
            Some(c) => Bool::and(&[&pc, c]),
            None => pc,
        };
        self.solver.reset();
        self.solver.assert(&query);
        self.solver_calls += 1;
        let t0 = Instant::now();
        let res = self.solver.check();
        self.t_check += t0.elapsed();
        if res != SatResult::Sat {
            return Ok(()); // no model: no candidate (the verdict is unchanged)
        }
        let model = self.solver.get_model().ok_or("cex-no-model")?;
        let mut nondets = Vec::with_capacity(st.nondets.len());
        for (name, bits, n, inst) in &st.nondets {
            let Some(sym) = self.symbols.get(n) else {
                continue;
            };
            let raw = st
                .pinned
                .get(n)
                .copied()
                .unwrap_or_else(|| model.eval(sym, true).and_then(|b| b.as_u64()).unwrap_or(0));
            let unsigned = ["uint", "ulong", "ushort", "uchar", "ulonglong", "size_t"]
                .iter()
                .any(|u| name.ends_with(u));
            let value: i64 = if *bits >= 64 || unsigned {
                i64::from_ne_bytes(raw.to_ne_bytes())
            } else if raw & (1u64 << (bits - 1)) != 0 {
                i64::from_ne_bytes(raw.to_ne_bytes()) - (1i64 << bits)
            } else {
                i64::from_ne_bytes(raw.to_ne_bytes())
            };
            nondets.push((name.clone(), value, *inst));
        }
        let block_path = st
            .path
            .iter()
            .filter(|(f, _)| *f == st.func)
            .map(|(_, b)| *b)
            .collect();
        trace(|| format!("counterexample: {} nondet reads", nondets.len()));
        self.cex = Some(Counterexample {
            nondets,
            error_inst: at,
            block_path,
        });
        Ok(())
    }

    /// The k-induction cases branch lazily: both are bounded (k iterations plus
    /// the exits), so a dead path costs at most that much before the header or
    /// error-call check kills it.
    fn lazy(&self) -> bool {
        match self.mode {
            Mode::Base { .. } | Mode::Step { .. } => true,
            Mode::Exhaustive => self.lazy_exhaustive,
        }
    }

    /// Where a lazy pass decides its path condition: the induction loop's
    /// header (induction cases), or any loop header (lazy exhaustive pass).
    fn at_lazy_checkpoint(&self, st: &State) -> bool {
        match self.mode {
            Mode::Exhaustive => self.loop_headers.contains(&(st.func, st.block)),
            Mode::Base { .. } | Mode::Step { .. } => self
                .kloop
                .as_ref()
                .is_some_and(|lp| lp.header == st.block && lp.func == st.func),
        }
    }

    /// Whole-path-condition check for the lazy modes: `Ok(false)` means the
    /// path is dead. `UNKNOWN` is kept as feasible and remembered.
    fn path_alive(&mut self, st: &mut State) -> Res<bool> {
        if st.constraints.len() == st.checked_len {
            return Ok(true);
        }
        let (pc, _) = path_condition(st);
        trace(|| {
            format!(
                "path_alive: func={} block={:?} visits={} constraints={}",
                self.module
                    .function(st.func)
                    .map_or("?", |f| f.name.as_str()),
                st.block,
                st.visits.get(&st.block).copied().unwrap_or(0),
                st.constraints.len()
            )
        });
        let alive = match self.feasible(st, &pc, &no_vars())? {
            Feas::Unsat => false,
            Feas::Sat => true,
            Feas::Unknown => {
                st.has_unknown = true;
                true
            }
        };
        st.checked_len = st.constraints.len();
        trace(|| format!("path_alive -> {alive}"));
        Ok(alive)
    }

    /// Whether to follow a branch guarded by `cond`. The base and exhaustive
    /// passes prune eagerly (one solver query per side). The step case does
    /// not: from an arbitrary header state nearly every branch is feasible, so
    /// the queries only burn the budget (measured: 95 % of the step case's time
    /// on ECA and `s3_srvr`). Instead the condition is added to the path and the
    /// body is encoded as one formula, decided at the error call — the BMC
    /// encoding k-induction tools use. Dead paths cost states, never soundness:
    /// an error reached on an infeasible path is refuted at the error call, and
    /// merging keeps the state count linear in the body.
    fn branch_feasible(&mut self, st: &mut State, cond: &Bool, vars: &Vars) -> Res<bool> {
        if self.lazy() {
            // A condition over at most two symbols (`state == k && input == j`
            // in an ECA body, where the state is concrete after a fixpoint
            // restart) is decided eagerly: the query sliced to its component is
            // tiny, and NOT deciding it let every dead branch of the state
            // machine run to the next join (14 k states per iteration).
            if !vars.is_empty() && vars.len() > 2 {
                return Ok(true);
            }
            if !vars.is_empty() {
                return self.feasible_or_unknown(st, cond, vars);
            }
            let t0 = Instant::now();
            let folded = cond.simplify().as_bool();
            self.t_simplify += t0.elapsed();
            return Ok(folded.unwrap_or(true));
        }
        self.feasible_or_unknown(st, cond, vars)
    }

    /// `true` iff `extra` is not refuted together with the path condition
    /// (`UNKNOWN` counts as feasible and is remembered on the state).
    fn feasible_or_unknown(&mut self, st: &mut State, extra: &Bool, vars: &Vars) -> Res<bool> {
        Ok(match self.feasible(st, extra, vars)? {
            Feas::Unsat => false,
            Feas::Sat => true,
            Feas::Unknown => {
                st.has_unknown = true;
                true
            }
        })
    }

    /// Is `extra` (over `vars`) satisfiable together with the path condition?
    /// Only the constraints connected to `vars` are sent (see [`Vars`]).
    fn feasible(&mut self, st: &State, extra: &Bool, vars: &Vars) -> Res<Feas> {
        // Constant-fold first: most loop and assert guards on a concrete path are
        // literal after simplification, and need no solver call.
        // Constant-fold a concrete query (its term is small); a symbolic one
        // goes straight to the solver — `simplify` on a large term costs more
        // than the query it would occasionally save.
        if vars.is_empty() {
            let t0 = Instant::now();
            let folded = extra.simplify().as_bool();
            self.t_simplify += t0.elapsed();
            if let Some(b) = folded {
                return Ok(if b { Feas::Sat } else { Feas::Unsat });
            }
        }
        // The connected component of the query's variables, to a fixpoint.
        let mut frontier: Vars = vars.clone();
        let mut chosen: Vec<usize> = Vec::new();
        let mut taken = vec![false; st.constraints.len()];
        loop {
            let mut grew = false;
            for (i, (_, cv)) in st.constraints.iter().enumerate() {
                if !taken[i] && !cv.is_disjoint(&frontier) {
                    taken[i] = true;
                    chosen.push(i);
                    let before = frontier.len();
                    frontier = union(&frontier, cv);
                    // `All` has the same `len` as `All`: compare identity too, or
                    // constraints earlier in the list are missed once the
                    // frontier saturates mid-pass (lost the XCSP proofs).
                    grew |= frontier.len() > before || matches!(*frontier, VarSet::All);
                }
            }
            if !grew {
                break;
            }
        }
        self.solver_calls += 1;
        if self.solver_calls > MAX_SOLVER_CALLS {
            return Err(format!("budget-solver{}", self.stats()));
        }
        self.solver.reset();
        for i in chosen {
            self.solver.assert(&st.constraints[i].0);
        }
        self.solver.assert(extra);
        if self.violation_query {
            // Whatever is left of the budget, up to the violation limit.
            let left = self
                .deadline
                .saturating_duration_since(Instant::now())
                .as_millis();
            let ms = u32::try_from(left)
                .unwrap_or(u32::MAX)
                .clamp(Z3_TIMEOUT_MS, Z3_VIOLATION_TIMEOUT_MS);
            self.params_violation.set_u32("timeout", ms);
            self.solver.set_params(&self.params_violation);
        } else {
            self.solver.set_params(&self.params);
        }
        let t0 = Instant::now();
        let res = self.solver.check();
        let took = t0.elapsed();
        self.t_check += took;
        // Experiment (`dumpquery`): write every query slower than 100 ms as
        // SMT-LIB 2 for inspection.
        if debug_flag("dumpquery") && took.as_millis() >= 100 {
            let dir = std::env::var("SAF_UNREACH_DUMP_DIR").unwrap_or_else(|_| "/tmp".into());
            let path = format!("{dir}/q{}_{}ms.smt2", self.solver_calls, took.as_millis());
            let _ = std::fs::write(path, self.solver.to_smt2());
        }
        if took.as_secs() >= 5 {
            trace(|| {
                format!(
                    "slow check: {took:?}, {} constraints asserted",
                    st.constraints.len()
                )
            });
        }
        Ok(match res {
            SatResult::Sat => Feas::Sat,
            SatResult::Unsat => Feas::Unsat,
            SatResult::Unknown => Feas::Unknown,
        })
    }
}

/// The program point a state is parked at, for merging: function, block,
/// instruction index, the call stack, and how often this block was entered.
type MergeKey = (
    FunctionId,
    BlockId,
    usize,
    Vec<(FunctionId, BlockId, usize)>,
    u32,
    bool,
);

/// The footprint of a parked state (see `header_fixpoint`): its concrete key
/// with every symbolic bitvector slot left as a placeholder, plus those slots'
/// terms. A bitvector counts as concrete when it simplifies to a numeral.
fn footprint(st: &State) -> (ConcreteKey, Vec<(usize, BV)>) {
    let mut vals: Vec<CVal> = Vec::new();
    let mut syms: Vec<(usize, BV)> = Vec::new();
    let push = |v: Option<&Val>, vals: &mut Vec<CVal>, syms: &mut Vec<(usize, BV)>| match v {
        None => vals.push(CVal::Absent),
        Some(Val::Ptr(p)) => vals.push(CVal::Ptr(p.clone())),
        Some(Val::Bv(bv, _)) => {
            if let Some(c) = bv.simplify().as_u64() {
                vals.push(CVal::Bv(c, bv.get_size()));
            } else {
                syms.push((vals.len(), bv.clone()));
                vals.push(CVal::Bv(0, bv.get_size()));
            }
        }
    };
    let mut frames = Vec::with_capacity(st.frames.len());
    for f in &st.frames {
        let mut saved_ids = Vec::with_capacity(f.saved.len());
        for (vid, v) in &f.saved {
            saved_ids.push(*vid);
            push(v.as_ref(), &mut vals, &mut syms);
        }
        frames.push((
            f.func,
            f.ret_block,
            f.ret_idx,
            f.ret_prev,
            f.ret_dst,
            saved_ids,
        ));
    }
    let value_ids: Vec<ValueId> = st.values.keys().copied().collect();
    for v in st.values.values() {
        push(Some(v), &mut vals, &mut syms);
    }
    let cells: Vec<Cell> = st.memory.keys().cloned().collect();
    for v in st.memory.values() {
        push(Some(v), &mut vals, &mut syms);
    }
    (
        ConcreteKey {
            func: st.func,
            block: st.block,
            idx: st.idx,
            frames,
            value_ids,
            cells,
            escaped: st.escaped.iter().copied().collect(),
            dead: st.dead.iter().copied().collect(),
            stack: st.stack.clone(),
            vals,
        },
        syms,
    )
}

/// The constraint-free state at `st`'s program point whose contents are the
/// concrete `key` (slot order as in [`footprint`]).
fn restart_state(st: &State, key: &ConcreteKey) -> State {
    let mut out = st.clone();
    let mut slots = key.vals.iter();
    let next = |slots: &mut std::slice::Iter<CVal>| -> Option<Val> {
        match slots.next() {
            None | Some(CVal::Absent) => None,
            Some(CVal::Ptr(p)) => Some(Val::Ptr(p.clone())),
            Some(CVal::Bv(c, bits)) => Some(Val::Bv(BV::from_u64(*c, *bits), no_vars())),
        }
    };
    for f in &mut out.frames {
        for (_, v) in &mut f.saved {
            *v = next(&mut slots);
        }
    }
    for v in out.values.values_mut() {
        if let Some(c) = next(&mut slots) {
            *v = c;
        }
    }
    for v in out.memory.values_mut() {
        if let Some(c) = next(&mut slots) {
            *v = c;
        }
    }
    out.constraints.clear();
    out.pc_cache = None;
    out.checked_len = 0;
    out.has_unknown = false;
    out.visits.clear();
    out.prev = None;
    out.parked = true;
    out.parked_here = true;
    out.restarted = true;
    out
}

fn merge_key(st: &State) -> MergeKey {
    (
        st.func,
        st.block,
        st.idx,
        st.frames
            .iter()
            .map(|f| (f.func, f.ret_block, f.ret_idx))
            .collect(),
        st.visits.get(&st.block).copied().unwrap_or(0),
        st.restarted,
    )
}

/// The conjunction of some constraints and the union of their variables.
fn conjunction(cs: &[(Bool, Vars)]) -> (Bool, Vars) {
    let bools: Vec<&Bool> = cs.iter().map(|(b, _)| b).collect();
    let pc = match bools.len() {
        0 => Bool::from_bool(true),
        1 => bools[0].clone(),
        _ => Bool::and(&bools),
    };
    let vars = cs.iter().fold(no_vars(), |acc, (_, v)| union(&acc, v));
    (pc, vars)
}

/// The conjunction of a state's path constraints and their variables.
fn path_condition(st: &mut State) -> (Bool, Vars) {
    let (done, mut pc, mut vars) = match st.pc_cache.take() {
        Some((n, pc, vars)) if n <= st.constraints.len() => (n, Some(pc), vars),
        _ => (0, None, no_vars()),
    };
    let rest: Vec<&Bool> = st.constraints[done..].iter().map(|(b, _)| b).collect();
    if !rest.is_empty() {
        let tail = Bool::and(&rest);
        pc = Some(match pc {
            Some(head) => Bool::and(&[&head, &tail]),
            None => tail,
        });
        for (_, v) in &st.constraints[done..] {
            vars = union(&vars, v);
        }
    }
    let pc = pc.unwrap_or_else(|| Bool::from_bool(true));
    st.pc_cache = Some((st.constraints.len(), pc.clone(), vars.clone()));
    (pc, vars)
}

/// The constant a global's initialiser holds at `path` (`[]` = the whole
/// initialiser): descend through aggregates by index; `ZeroInit`/`Undef` are
/// themselves at every path; a scalar below a non-empty path, or an index
/// outside the aggregate, has no defined value (`None` → the load is fresh).
/// The initialiser's constant at a structural `path`: an aggregate's element,
/// a zero/undef initialiser at any depth, or one byte of a `char` array the
/// frontend recorded as text (past the recorded text — which may stop at an
/// interior NUL — the byte is unknown, so `None`).
fn init_at_path(init: &Constant, path: &[u32]) -> Option<Constant> {
    let mut cur = init;
    for (depth, &i) in path.iter().enumerate() {
        cur = match cur {
            Constant::Aggregate { elements } => elements.get(usize::try_from(i).ok()?)?,
            Constant::ZeroInit | Constant::Undef => return Some(cur.clone()),
            Constant::String { value } if depth + 1 == path.len() => {
                let byte = *value.as_bytes().get(usize::try_from(i).ok()?)?;
                return Some(Constant::Int {
                    value: i64::from(byte),
                    bits: 8,
                });
            }
            _ => return None,
        };
    }
    // A cell path is canonical — trailing zero indices stripped (`a[0]` IS `a`,
    // the first field IS the struct) — so a path ending at an aggregate names
    // its first scalar.
    loop {
        match cur {
            Constant::Aggregate { elements } => cur = elements.first()?,
            Constant::String { value } => {
                let byte = *value.as_bytes().first()?;
                return Some(Constant::Int {
                    value: i64::from(byte),
                    bits: 8,
                });
            }
            _ => return Some(cur.clone()),
        }
    }
}

enum Flow {
    Next,
    Jump(BlockId),
    Fork(Vec<(BlockId, Bool, Vars)>),
    /// Split the state on these mutually exclusive, jointly exhaustive
    /// conditions (each pins a symbol to one value) and retry the instruction.
    Refine(Vec<(Bool, Vars)>),
    Return(Option<Val>),
    Halt,
}

/// The condition under which `a <kind> b` is a signed overflow (see
/// [`Goal::SignedOverflow`]); `None` when the operation cannot be one.
fn overflow_condition(inst: &Instruction, kind: BinaryOp, a: &BV, b: &BV) -> Option<Bool> {
    let bits = a.get_size();
    Some(match kind {
        // Signed add/sub/mul carry `nsw`; unsigned ones do not (defined wrap).
        // Overflow ⇔ the result in a wider type differs from the wrapped one.
        BinaryOp::Add | BinaryOp::Sub if inst.has_no_signed_wrap() => {
            let (wa, wb) = (a.sign_ext(1), b.sign_ext(1));
            let (wide, narrow) = if kind == BinaryOp::Add {
                (wa.bvadd(&wb), a.bvadd(b))
            } else {
                (wa.bvsub(&wb), a.bvsub(b))
            };
            eq(&wide, &narrow.sign_ext(1)).not()
        }
        // Two equivalent encodings, conjoined: the 2·bits-wide product against
        // the sign-extended narrow one (what nla-digbench's products need) and
        // Z3's `bvmul_no_overflow` predicates (what a guarded `x * x` square
        // needs — alone, each loses the other's proofs to the budget). An
        // UNSAT answer only needs the refutation the solver finds first.
        BinaryOp::Mul if inst.has_no_signed_wrap() => {
            // A square `x * x` overflows iff |x| > floor(sqrt(MAX)): one
            // comparison against a constant instead of a multiplier circuit
            // the SAT solver cannot close for 64 bits (Juliet's `*_square`).
            if a.get_z3_ast() == b.get_z3_ast() {
                let max = i128::from(i64::MAX >> (64 - bits));
                let bound = isqrt(max);
                let hi = int_bv(bound, bits);
                let lo = int_bv(-bound, bits);
                return Some(Bool::or(&[&a.bvsgt(&hi), &a.bvslt(&lo)]));
            }
            let by_predicate =
                Bool::and(&[&a.bvmul_no_overflow(b, true), &a.bvmul_no_underflow(b)]).not();
            if debug_flag("mulpred") {
                return Some(by_predicate);
            }
            let wide = a.sign_ext(bits).bvmul(b.sign_ext(bits));
            let by_product = eq(&wide, &a.bvmul(b).sign_ext(bits)).not();
            if debug_flag("mulwide") {
                return Some(by_product);
            }
            Bool::and(&[&by_product, &by_predicate])
        }
        // `sdiv`/`srem` exist only for signed operands: `INT_MIN / -1`.
        BinaryOp::SDiv | BinaryOp::SRem => {
            let int_min = int_bv(1, bits).bvshl(int_bv(i64::from(bits) - 1, bits));
            Bool::and(&[&eq(a, &int_min), &eq(b, &int_bv(-1, bits))])
        }
        // The IR does not say whether a shift's operand was signed, so every
        // left shift is held to the signed rule the property uses: the
        // mathematical `a · 2^b` must be representable (a negative `a` is fine
        // when it is — `(int64_t) rand() << 32` is TRUE in Juliet). Shifting
        // back must give `a`; a shift by ≥ width yields 0, caught unless `a == 0`.
        BinaryOp::Shl => eq(&a.bvshl(b).bvashr(b), a).not(),
        _ => return None,
    })
}

/// `cond` simplifies to `s<n> == K` for a fresh symbol and a numeral:
/// `(n, symbol, numeral)`.
fn symbol_equality(cond: &Bool) -> Option<(u64, BV, BV)> {
    let simple = cond.simplify();
    if !simple.is_app() || simple.decl().kind() != z3::DeclKind::EQ || simple.num_children() != 2 {
        return None;
    }
    let (lhs, rhs) = (simple.nth_child(0)?.as_bv()?, simple.nth_child(1)?.as_bv()?);
    let (sym, k) = if lhs.as_u64().is_some() {
        (rhs, lhs)
    } else {
        (lhs, rhs)
    };
    if k.as_u64().is_none() || !sym.is_const() || sym.decl().kind() != z3::DeclKind::UNINTERPRETED {
        return None;
    }
    let n: u64 = sym.decl().name().strip_prefix('s')?.parse().ok()?;
    Some((n, sym, k))
}

/// IEEE-754 semantics for the engine's float values, which travel as the
/// bitvector of their encoding (`float` 32 bits, `double` 64). Every operation
/// converts to Z3's floating-point sort, computes with round-to-nearest-even
/// (C's default mode), and converts back with `fp.to_ieee_bv`. A NaN result
/// takes whichever NaN encoding Z3 chooses — all a C program can observe of a
/// NaN through `==`, `<` or `isnan` is that it is one. The z3 crate exposes no
/// FPA arithmetic, so this goes through the raw bindings; every intermediate is
/// wrapped (reference-counted) before the next call.
// NOTE: `c`, `x`, `y`, `s` are the names Z3's own API uses for the context,
// operands and sort; longer names would only obscure the FFI calls.
#[allow(clippy::many_single_char_names)]
mod fpa {
    use super::Res;
    use saf_core::air::BinaryOp;
    use z3::Context;
    use z3::ast::{Ast, BV, Bool, Dynamic, Float};
    use z3_sys::{
        Z3_ast, Z3_context, Z3_get_error_code, Z3_get_error_msg, Z3_inc_ref, Z3_mk_fpa_abs,
        Z3_mk_fpa_add, Z3_mk_fpa_div, Z3_mk_fpa_eq, Z3_mk_fpa_fma, Z3_mk_fpa_geq, Z3_mk_fpa_gt,
        Z3_mk_fpa_is_nan, Z3_mk_fpa_leq, Z3_mk_fpa_lt, Z3_mk_fpa_max, Z3_mk_fpa_min, Z3_mk_fpa_mul,
        Z3_mk_fpa_numeral_double, Z3_mk_fpa_rna, Z3_mk_fpa_rne, Z3_mk_fpa_round_to_integral,
        Z3_mk_fpa_rtn, Z3_mk_fpa_rtp, Z3_mk_fpa_rtz, Z3_mk_fpa_sort, Z3_mk_fpa_sort_32,
        Z3_mk_fpa_sort_64, Z3_mk_fpa_sqrt, Z3_mk_fpa_sub, Z3_mk_fpa_to_fp_bv,
        Z3_mk_fpa_to_fp_float, Z3_mk_fpa_to_fp_signed, Z3_mk_fpa_to_fp_unsigned,
        Z3_mk_fpa_to_ieee_bv, Z3_mk_fpa_to_sbv, Z3_mk_fpa_to_ubv, Z3_sort, Z3_sort_to_ast,
    };

    /// A raw result, or Z3's own error message (`z3-fpa: <message>`) when the
    /// call returned null — the message names the sort mismatch.
    fn ok(c: Z3_context, a: Option<Z3_ast>) -> Res<Z3_ast> {
        a.ok_or_else(|| {
            // SAFETY: reading the context's last error on a live context.
            let msg = unsafe {
                let code = Z3_get_error_code(c);
                std::ffi::CStr::from_ptr(Z3_get_error_msg(c, code))
                    .to_string_lossy()
                    .into_owned()
            };
            format!("z3-fpa: {msg}")
        })
    }

    /// The FPA sort of a `bits`-wide encoding: `float`, `double`, or the x87
    /// 80-bit `long double` as Z3's (15, 64) sort — same exponent and the same
    /// 64-bit precision; the x87 encoding's explicit integer bit is dropped on
    /// the way in and recomputed on the way out (`of_bits`/`to_bits`). The
    /// context is reference-counted: a sort left at zero references is freed
    /// by the next allocation (seen as "rm and float sorts expected" from
    /// `fp.to_fp`), so the sort is pinned once per use — sorts are interned,
    /// so this only bumps one counter.
    fn sort(c: Z3_context, bits: u32) -> Res<Z3_sort> {
        // SAFETY: plain constructors on a live context; the sort is pinned.
        let s = unsafe {
            match bits {
                32 => Z3_mk_fpa_sort_32(c),
                64 => Z3_mk_fpa_sort_64(c),
                80 => Z3_mk_fpa_sort(c, 15, 64),
                _ => None,
            }
        };
        let s = s.ok_or_else(|| "fp-width".to_string())?;
        if let Some(pin) = unsafe { Z3_sort_to_ast(c, s) } {
            unsafe { Z3_inc_ref(c, pin) };
        }
        Ok(s)
    }

    fn rounding(cx: &Context, nearest: bool) -> Res<Dynamic> {
        rounding_mode(cx, if nearest { "rne" } else { "rtz" })
    }

    /// One of the five IEEE rounding modes by its SMT-LIB name.
    fn rounding_mode(cx: &Context, mode: &str) -> Res<Dynamic> {
        let c = cx.get_z3_context();
        // SAFETY: plain constructors on a live context; the result is wrapped
        // (reference-counted) before anything else runs.
        let a = ok(c, unsafe {
            match mode {
                "rne" => Z3_mk_fpa_rne(c),
                "rtz" => Z3_mk_fpa_rtz(c),
                "rtn" => Z3_mk_fpa_rtn(c),
                "rtp" => Z3_mk_fpa_rtp(c),
                "rna" => Z3_mk_fpa_rna(c),
                _ => None,
            }
        })?;
        Ok(unsafe { Dynamic::wrap(cx, a) })
    }

    /// `fabs`, `sqrt` (correctly rounded, as libm's) and the rounding-to-integer
    /// family (`floor` RTN, `ceil` RTP, `trunc` RTZ, `round` RNA, `rint` RNE).
    pub fn unary(family: &str, a: &BV) -> Res<BV> {
        let cx = Context::thread_local();
        let c = cx.get_z3_context();
        let x = of_bits(&cx, a)?;
        // SAFETY: live operands; wrapped result.
        let r = match family {
            "fabs" => ok(c, unsafe { Z3_mk_fpa_abs(c, x.get_z3_ast()) })?,
            "sqrt" => {
                let rm = rounding(&cx, true)?;
                ok(c, unsafe {
                    Z3_mk_fpa_sqrt(c, rm.get_z3_ast(), x.get_z3_ast())
                })?
            }
            _ => {
                let mode = match family {
                    "floor" => "rtn",
                    "ceil" => "rtp",
                    "trunc" => "rtz",
                    "round" => "rna",
                    _ => "rne",
                };
                let rm = rounding_mode(&cx, mode)?;
                ok(c, unsafe {
                    Z3_mk_fpa_round_to_integral(c, rm.get_z3_ast(), x.get_z3_ast())
                })?
            }
        };
        let r = unsafe { Float::wrap(&cx, r) };
        to_bits(&cx, &r)
    }

    /// `fmin`/`fmax`: the other operand when one is NaN, as C specifies.
    pub fn minmax(min: bool, a: &BV, b: &BV) -> Res<BV> {
        let cx = Context::thread_local();
        let c = cx.get_z3_context();
        let (x, y) = (of_bits(&cx, a)?, of_bits(&cx, b)?);
        // SAFETY: live operands; wrapped result.
        let r = ok(c, unsafe {
            if min {
                Z3_mk_fpa_min(c, x.get_z3_ast(), y.get_z3_ast())
            } else {
                Z3_mk_fpa_max(c, x.get_z3_ast(), y.get_z3_ast())
            }
        })?;
        let r = unsafe { Float::wrap(&cx, r) };
        to_bits(&cx, &r)
    }

    /// Fused `a * b + c` with a single rounding.
    pub fn fma(a: &BV, b: &BV, d: &BV) -> Res<BV> {
        let cx = Context::thread_local();
        let c = cx.get_z3_context();
        let (x, y, z) = (of_bits(&cx, a)?, of_bits(&cx, b)?, of_bits(&cx, d)?);
        let rm = rounding(&cx, true)?;
        // SAFETY: live operands; wrapped result.
        let r = ok(c, unsafe {
            Z3_mk_fpa_fma(
                c,
                rm.get_z3_ast(),
                x.get_z3_ast(),
                y.get_z3_ast(),
                z.get_z3_ast(),
            )
        })?;
        let r = unsafe { Float::wrap(&cx, r) };
        to_bits(&cx, &r)
    }

    fn of_bits(cx: &Context, bv: &BV) -> Res<Float> {
        let c = cx.get_z3_context();
        let s = sort(c, bv.get_size())?;
        // x87: sign(79) exponent(78..64) INTEGER BIT(63) fraction(62..0); the
        // integer bit is implied by the exponent in an IEEE-style encoding.
        let packed = if bv.get_size() == 80 {
            bv.extract(79, 64).concat(bv.extract(62, 0))
        } else {
            bv.clone()
        };
        // SAFETY: `packed` is a live AST of the same context; the result is wrapped.
        let a = ok(c, unsafe { Z3_mk_fpa_to_fp_bv(c, packed.get_z3_ast(), s) })?;
        Ok(unsafe { Float::wrap(cx, a) })
    }

    fn to_bits(cx: &Context, f: &Float) -> Res<BV> {
        // SAFETY: as above.
        let a = ok(cx.get_z3_context(), unsafe {
            Z3_mk_fpa_to_ieee_bv(cx.get_z3_context(), f.get_z3_ast())
        })?;
        let bv = unsafe { BV::wrap(cx, a) };
        if bv.get_size() == 79 {
            // Back to x87: the integer bit is 1 unless the number is zero or
            // subnormal (exponent field 0).
            let exp = bv.extract(77, 63);
            let int_bit = super::eq(&exp, &super::int_bv(0, 15))
                .ite(&super::int_bv(0, 1), &super::int_bv(1, 1));
            return Ok(bv
                .extract(78, 63)
                .concat(&int_bit)
                .concat(bv.extract(62, 0)));
        }
        Ok(bv)
    }

    fn numeral(cx: &Context, value: f64, bits: u32) -> Res<Float> {
        let c = cx.get_z3_context();
        let s = sort(c, bits)?;
        // SAFETY: plain constructor; wrapped.
        let a = ok(cx.get_z3_context(), unsafe {
            Z3_mk_fpa_numeral_double(c, value, s)
        })?;
        Ok(unsafe { Float::wrap(cx, a) })
    }

    fn is_nan(cx: &Context, f: &Float) -> Res<Bool> {
        // SAFETY: as above.
        let a = ok(cx.get_z3_context(), unsafe {
            Z3_mk_fpa_is_nan(cx.get_z3_context(), f.get_z3_ast())
        })?;
        Ok(unsafe { Bool::wrap(cx, a) })
    }

    type Pred = unsafe extern "C" fn(Z3_context, Z3_ast, Z3_ast) -> Option<Z3_ast>;

    fn pred(cx: &Context, f: Pred, x: &Float, y: &Float) -> Res<Bool> {
        // SAFETY: both operands are live ASTs of this context; wrapped.
        let a = ok(cx.get_z3_context(), unsafe {
            f(cx.get_z3_context(), x.get_z3_ast(), y.get_z3_ast())
        })?;
        Ok(unsafe { Bool::wrap(cx, a) })
    }

    /// `a ∘ b` for `+ - * /`, round-to-nearest-even.
    pub fn arith(kind: BinaryOp, a: &BV, b: &BV) -> Res<BV> {
        let cx = Context::thread_local();
        let c = cx.get_z3_context();
        let (x, y) = (of_bits(&cx, a)?, of_bits(&cx, b)?);
        let rm = rounding(&cx, true)?;
        // SAFETY: all operands live and wrapped; the result is wrapped.
        let r = unsafe {
            let (rm, x, y) = (rm.get_z3_ast(), x.get_z3_ast(), y.get_z3_ast());
            match kind {
                BinaryOp::FAdd => Z3_mk_fpa_add(c, rm, x, y),
                BinaryOp::FSub => Z3_mk_fpa_sub(c, rm, x, y),
                BinaryOp::FMul => Z3_mk_fpa_mul(c, rm, x, y),
                BinaryOp::FDiv => Z3_mk_fpa_div(c, rm, x, y),
                _ => None,
            }
        };
        let r = unsafe { Float::wrap(&cx, ok(c, r)?) };
        to_bits(&cx, &r)
    }

    /// An `fcmp` predicate. The ordered forms are false on a NaN operand, the
    /// unordered ones true; `fp.eq` is IEEE equality (`-0 == +0`, NaN ≠ NaN).
    pub fn cmp(kind: BinaryOp, a: &BV, b: &BV) -> Res<Bool> {
        let cx = Context::thread_local();
        let (x, y) = (of_bits(&cx, a)?, of_bits(&cx, b)?);
        let unordered = Bool::or(&[&is_nan(&cx, &x)?, &is_nan(&cx, &y)?]);
        let p = |f: Pred| pred(&cx, f, &x, &y);
        Ok(match kind {
            BinaryOp::FCmpOeq => p(Z3_mk_fpa_eq)?,
            BinaryOp::FCmpOgt => p(Z3_mk_fpa_gt)?,
            BinaryOp::FCmpOge => p(Z3_mk_fpa_geq)?,
            BinaryOp::FCmpOlt => p(Z3_mk_fpa_lt)?,
            BinaryOp::FCmpOle => p(Z3_mk_fpa_leq)?,
            BinaryOp::FCmpOne => Bool::and(&[&unordered.not(), &p(Z3_mk_fpa_eq)?.not()]),
            BinaryOp::FCmpUeq => Bool::or(&[&unordered, &p(Z3_mk_fpa_eq)?]),
            BinaryOp::FCmpUgt => Bool::or(&[&unordered, &p(Z3_mk_fpa_gt)?]),
            BinaryOp::FCmpUge => Bool::or(&[&unordered, &p(Z3_mk_fpa_geq)?]),
            BinaryOp::FCmpUlt => Bool::or(&[&unordered, &p(Z3_mk_fpa_lt)?]),
            BinaryOp::FCmpUle => Bool::or(&[&unordered, &p(Z3_mk_fpa_leq)?]),
            BinaryOp::FCmpUne => p(Z3_mk_fpa_eq)?.not(),
            BinaryOp::FCmpOrd => unordered.not(),
            BinaryOp::FCmpUno => unordered,
            BinaryOp::FCmpTrue => Bool::from_bool(true),
            BinaryOp::FCmpFalse => Bool::from_bool(false),
            _ => return Err(format!("fcmp-{kind:?}")),
        })
    }

    /// `(float)i` / `(double)i`, round-to-nearest-even.
    pub fn from_int(a: &BV, signed: bool, width: u32) -> Res<BV> {
        let cx = Context::thread_local();
        let c = cx.get_z3_context();
        let s = sort(c, width)?;
        let rm = rounding(&cx, true)?;
        // SAFETY: live operands; wrapped result.
        let r = ok(cx.get_z3_context(), unsafe {
            if signed {
                Z3_mk_fpa_to_fp_signed(c, rm.get_z3_ast(), a.get_z3_ast(), s)
            } else {
                Z3_mk_fpa_to_fp_unsigned(c, rm.get_z3_ast(), a.get_z3_ast(), s)
            }
        })?;
        let r = unsafe { Float::wrap(&cx, r) };
        to_bits(&cx, &r)
    }

    /// `fpext` / `fptrunc` (round-to-nearest-even).
    pub fn convert(a: &BV, width: u32) -> Res<BV> {
        let cx = Context::thread_local();
        let c = cx.get_z3_context();
        let x = of_bits(&cx, a)?;
        let s = sort(c, width)?;
        let rm = rounding(&cx, true)?;
        // SAFETY: live operands; wrapped result.
        let r = ok(cx.get_z3_context(), unsafe {
            Z3_mk_fpa_to_fp_float(c, rm.get_z3_ast(), x.get_z3_ast(), s)
        })?;
        let r = unsafe { Float::wrap(&cx, r) };
        to_bits(&cx, &r)
    }

    /// `(int)x` truncating toward zero, and the condition under which that is
    /// defined: `x` is not NaN and `lo < x < hi` for the target's range (the
    /// bounds are the nearest representable floats, so a value exactly at the
    /// edge of the range may be reported out of range — a sound loss).
    pub fn to_int(a: &BV, signed: bool, width: u32) -> Res<(BV, Bool)> {
        let cx = Context::thread_local();
        let c = cx.get_z3_context();
        let x = of_bits(&cx, a)?;
        let rm = rounding(&cx, false)?;
        // SAFETY: live operands; wrapped result.
        let r = ok(cx.get_z3_context(), unsafe {
            if signed {
                Z3_mk_fpa_to_sbv(c, rm.get_z3_ast(), x.get_z3_ast(), width)
            } else {
                Z3_mk_fpa_to_ubv(c, rm.get_z3_ast(), x.get_z3_ast(), width)
            }
        })?;
        let conv = unsafe { BV::wrap(&cx, r) };
        let exp = i32::try_from(width).map_err(|_| "fp-width".to_string())?;
        let (lo, hi) = if signed {
            let half = 2f64.powi(exp - 1);
            (-half - 1.0, half)
        } else {
            (-1.0, 2f64.powi(exp))
        };
        let (lo, hi) = (
            numeral(&cx, lo, a.get_size())?,
            numeral(&cx, hi, a.get_size())?,
        );
        let in_range = Bool::and(&[
            &is_nan(&cx, &x)?.not(),
            &pred(&cx, Z3_mk_fpa_lt, &lo, &x)?,
            &pred(&cx, Z3_mk_fpa_lt, &x, &hi)?,
        ]);
        Ok((conv, in_range))
    }
}

/// floor(sqrt(n)) for a non-negative `n` (the overflow bound of a square).
fn isqrt(n: i128) -> i64 {
    let mut r = 0i128;
    let mut bit = 1i128 << 62;
    let mut rem = n;
    while bit != 0 {
        if rem >= r + bit {
            rem -= r + bit;
            r = (r >> 1) + bit;
        } else {
            r >>= 1;
        }
        bit >>= 2;
    }
    i64::try_from(r).unwrap_or(i64::MAX)
}

fn int_bv(value: i64, bits: u32) -> BV {
    if bits >= 64 {
        BV::from_i64(value, bits)
    } else {
        // Two's-complement truncation of the i64 literal to `bits`.
        #[allow(clippy::cast_sign_loss)]
        let masked = (value as u64) & ((1u64 << bits) - 1);
        BV::from_u64(masked, bits)
    }
}

fn eq(a: &BV, b: &BV) -> Bool {
    #[allow(deprecated)]
    a._eq(b)
}

fn truthy(v: &BV) -> Bool {
    eq(v, &int_bv(0, v.get_size())).not()
}

#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::air::{AirBlock, AirFunction, AirParam, Constant};
    use saf_core::ids::{FunctionId, InstId, ModuleId, TypeId};

    const I32: TypeId = TypeId::new(32);
    const I1: TypeId = TypeId::new(1);
    const PTR: TypeId = TypeId::new(7);
    const F64: TypeId = TypeId::new(640);
    const F32: TypeId = TypeId::new(320);

    struct B {
        m: AirModule,
        next: u128,
    }

    impl B {
        fn new() -> Self {
            let mut m = AirModule::new(ModuleId::new(1));
            m.types.insert(I32, AirType::Integer { bits: 32 });
            m.types.insert(I1, AirType::Integer { bits: 1 });
            m.types.insert(PTR, AirType::Pointer);
            m.types.insert(F64, AirType::Float { bits: 64 });
            m.types.insert(F32, AirType::Float { bits: 32 });
            // An exact frontend record: no constant pointer expression collapsed.
            m.collapsed_globals = Some(BTreeSet::new());
            Self { m, next: 1000 }
        }
        fn id(&mut self) -> u128 {
            self.next += 1;
            self.next
        }
        fn konst(&mut self, v: i64) -> ValueId {
            let id = ValueId::new(self.id());
            self.m
                .constants
                .insert(id, Constant::Int { value: v, bits: 32 });
            id
        }
        fn fkonst(&mut self, v: f64) -> ValueId {
            let id = ValueId::new(self.id());
            self.m
                .constants
                .insert(id, Constant::Float { value: v, bits: 64 });
            id
        }
        fn decl(&mut self, name: &str) -> FunctionId {
            let id = FunctionId::new(self.id());
            self.m.functions.push(AirFunction {
                id,
                name: name.into(),
                params: vec![],
                blocks: vec![],
                entry_block: None,
                is_declaration: true,
                span: None,
                symbol: None,
                block_index: BTreeMap::new(),
            });
            id
        }
        fn func(&mut self, name: &str, params: Vec<AirParam>, blocks: Vec<AirBlock>) -> FunctionId {
            let id = FunctionId::new(self.id());
            let entry = blocks.first().map(|b| b.id);
            self.m.functions.push(AirFunction {
                id,
                name: name.into(),
                params,
                blocks,
                entry_block: entry,
                is_declaration: false,
                span: None,
                symbol: None,
                block_index: BTreeMap::new(),
            });
            id
        }
        fn inst(
            &mut self,
            op: Operation,
            operands: Vec<ValueId>,
            ty: Option<TypeId>,
        ) -> (Instruction, Option<ValueId>) {
            let dst = ty.map(|_| ValueId::new(self.id()));
            let mut i = Instruction::new(InstId::new(self.id()), op);
            i.operands = operands;
            i.dst = dst;
            i.result_type = ty;
            (i, dst)
        }
        fn bid(&mut self) -> BlockId {
            BlockId::new(self.id())
        }
    }

    /// Attach the `GEP_BYTES_KEY` byte arithmetic the LLVM frontend records.
    fn gep_bytes(inst: &mut Instruction, steps: &[(bool, u64)]) {
        inst.extensions.insert(
            GEP_BYTES_KEY.to_string(),
            serde_json::Value::Array(
                steps
                    .iter()
                    .map(|(f, n)| serde_json::json!([u8::from(*f), *n]))
                    .collect(),
            ),
        );
    }

    fn point_span(line: u32, col: u32) -> saf_core::span::Span {
        saf_core::span::Span::new(
            saf_core::ids::FileId::new(1),
            0,
            1,
            line,
            col,
            line,
            col + 1,
        )
    }

    fn blk(id: BlockId, insts: Vec<Instruction>) -> AirBlock {
        AirBlock {
            id,
            label: None,
            instructions: insts,
        }
    }

    fn prove(m: &AirModule) -> ExhaustiveProof {
        prove_unreach_exhaustive(m, Duration::from_secs(30))
    }

    /// `int x = nondet(); if (x > 5 && x < 3) reach_error();` — the guard is
    /// contradictory, so the error is unreachable.
    #[test]
    fn contradictory_guard_is_proven() {
        let mut b = B::new();
        let err = b.decl("reach_error");
        let nd = b.decl("__VERIFIER_nondet_int");
        let (e, bb_then, bb_end) = (b.bid(), b.bid(), b.bid());
        let (i_nd, x) = b.inst(Operation::CallDirect { callee: nd }, vec![], Some(I32));
        let five = b.konst(5);
        let three = b.konst(3);
        let (i_gt, gt) = b.inst(
            Operation::BinaryOp {
                kind: BinaryOp::ICmpSgt,
            },
            vec![x.unwrap(), five],
            Some(I1),
        );
        let (i_lt, lt) = b.inst(
            Operation::BinaryOp {
                kind: BinaryOp::ICmpSlt,
            },
            vec![x.unwrap(), three],
            Some(I1),
        );
        let (i_and, both) = b.inst(
            Operation::BinaryOp {
                kind: BinaryOp::And,
            },
            vec![gt.unwrap(), lt.unwrap()],
            Some(I1),
        );
        let (i_br, _) = b.inst(
            Operation::CondBr {
                then_target: bb_then,
                else_target: bb_end,
            },
            vec![both.unwrap()],
            None,
        );
        let (i_err, _) = b.inst(Operation::CallDirect { callee: err }, vec![], None);
        let (i_ret1, _) = b.inst(Operation::Ret, vec![], None);
        let (i_ret2, _) = b.inst(Operation::Ret, vec![], None);
        b.func(
            "main",
            vec![],
            vec![
                blk(e, vec![i_nd, i_gt, i_lt, i_and, i_br]),
                blk(bb_then, vec![i_err, i_ret1]),
                blk(bb_end, vec![i_ret2]),
            ],
        );
        assert_eq!(prove(&b.m), ExhaustiveProof::Proven);
    }

    /// `if (x > 5) reach_error();` with nondet `x` — feasible: never a proof.
    #[test]
    fn feasible_error_abstains() {
        let mut b = B::new();
        let err = b.decl("reach_error");
        let nd = b.decl("__VERIFIER_nondet_int");
        let (e, bb_then, bb_end) = (b.bid(), b.bid(), b.bid());
        let (i_nd, x) = b.inst(Operation::CallDirect { callee: nd }, vec![], Some(I32));
        let five = b.konst(5);
        let (i_gt, gt) = b.inst(
            Operation::BinaryOp {
                kind: BinaryOp::ICmpSgt,
            },
            vec![x.unwrap(), five],
            Some(I1),
        );
        let (i_br, _) = b.inst(
            Operation::CondBr {
                then_target: bb_then,
                else_target: bb_end,
            },
            vec![gt.unwrap()],
            None,
        );
        let (i_err, _) = b.inst(Operation::CallDirect { callee: err }, vec![], None);
        let (i_ret1, _) = b.inst(Operation::Ret, vec![], None);
        let (i_ret2, _) = b.inst(Operation::Ret, vec![], None);
        b.func(
            "main",
            vec![],
            vec![
                blk(e, vec![i_nd, i_gt, i_br]),
                blk(bb_then, vec![i_err, i_ret1]),
                blk(bb_end, vec![i_ret2]),
            ],
        );
        assert_eq!(
            prove(&b.m),
            ExhaustiveProof::Abstain("error-feasible".into())
        );
    }

    /// `unsigned p = 1; int m = -1; if (p > m) reach_error();` — `m` converts to
    /// `0xFFFFFFFF`, so the unsigned compare is false: proven. The interval
    /// sentinel abstains on exactly this (bitvector-regression).
    #[test]
    fn unsigned_compare_of_negative_is_exact() {
        let mut b = B::new();
        let err = b.decl("reach_error");
        let (e, bb_then, bb_end) = (b.bid(), b.bid(), b.bid());
        let one = b.konst(1);
        let minus_one = b.konst(-1);
        let (i_cmp, c) = b.inst(
            Operation::BinaryOp {
                kind: BinaryOp::ICmpUgt,
            },
            vec![one, minus_one],
            Some(I1),
        );
        let (i_br, _) = b.inst(
            Operation::CondBr {
                then_target: bb_then,
                else_target: bb_end,
            },
            vec![c.unwrap()],
            None,
        );
        let (i_err, _) = b.inst(Operation::CallDirect { callee: err }, vec![], None);
        let (i_ret1, _) = b.inst(Operation::Ret, vec![], None);
        let (i_ret2, _) = b.inst(Operation::Ret, vec![], None);
        b.func(
            "main",
            vec![],
            vec![
                blk(e, vec![i_cmp, i_br]),
                blk(bb_then, vec![i_err, i_ret1]),
                blk(bb_end, vec![i_ret2]),
            ],
        );
        assert_eq!(prove(&b.m), ExhaustiveProof::Proven);
    }

    /// `x = 1; y = 0; while (y < 1024) { x = 0; y++; } if (x != 0) reach_error();`
    /// — needs the loop unwound to completion (1024 iterations).
    #[test]
    fn bounded_loop_is_unwound_to_completion() {
        let mut b = B::new();
        let err = b.decl("reach_error");
        let (e, hdr, body, exit, bb_then, bb_end) =
            (b.bid(), b.bid(), b.bid(), b.bid(), b.bid(), b.bid());
        let (zero, one, k1024) = (b.konst(0), b.konst(1), b.konst(1024));
        let (i_br0, _) = b.inst(Operation::Br { target: hdr }, vec![], None);
        // header: phis for x, y
        let x_phi_dst = ValueId::new(b.id());
        let y_phi_dst = ValueId::new(b.id());
        let (mut i_xphi, _) = b.inst(Operation::Phi { incoming: vec![] }, vec![], Some(I32));
        let (mut i_yphi, _) = b.inst(Operation::Phi { incoming: vec![] }, vec![], Some(I32));
        i_xphi.dst = Some(x_phi_dst);
        i_yphi.dst = Some(y_phi_dst);
        let (i_lt, lt) = b.inst(
            Operation::BinaryOp {
                kind: BinaryOp::ICmpSlt,
            },
            vec![y_phi_dst, k1024],
            Some(I1),
        );
        let (i_br1, _) = b.inst(
            Operation::CondBr {
                then_target: body,
                else_target: exit,
            },
            vec![lt.unwrap()],
            None,
        );
        let (i_inc, y2) = b.inst(
            Operation::BinaryOp {
                kind: BinaryOp::Add,
            },
            vec![y_phi_dst, one],
            Some(I32),
        );
        let (i_br2, _) = b.inst(Operation::Br { target: hdr }, vec![], None);
        i_xphi.op = Operation::Phi {
            incoming: vec![(e, one), (body, zero)],
        };
        i_yphi.op = Operation::Phi {
            incoming: vec![(e, zero), (body, y2.unwrap())],
        };
        let (i_ne, ne) = b.inst(
            Operation::BinaryOp {
                kind: BinaryOp::ICmpNe,
            },
            vec![x_phi_dst, zero],
            Some(I1),
        );
        let (i_br3, _) = b.inst(
            Operation::CondBr {
                then_target: bb_then,
                else_target: bb_end,
            },
            vec![ne.unwrap()],
            None,
        );
        let (i_err, _) = b.inst(Operation::CallDirect { callee: err }, vec![], None);
        let (i_ret1, _) = b.inst(Operation::Ret, vec![], None);
        let (i_ret2, _) = b.inst(Operation::Ret, vec![], None);
        b.func(
            "main",
            vec![],
            vec![
                blk(e, vec![i_br0]),
                blk(hdr, vec![i_xphi, i_yphi, i_lt, i_br1]),
                blk(body, vec![i_inc, i_br2]),
                blk(exit, vec![i_ne, i_br3]),
                blk(bb_then, vec![i_err, i_ret1]),
                blk(bb_end, vec![i_ret2]),
            ],
        );
        assert_eq!(prove(&b.m), ExhaustiveProof::Proven);
    }

    /// An unbounded loop (`while (nondet())`) exceeds the visit cap of the
    /// exhaustive pass; k-induction then proves it (there is no error call at
    /// all, so the step case holds for k = 1).
    #[test]
    fn unbounded_loop_without_error_is_proven_by_induction() {
        let mut b = B::new();
        let nd = b.decl("__VERIFIER_nondet_int");
        let (e, hdr, body, exit) = (b.bid(), b.bid(), b.bid(), b.bid());
        let (i_br0, _) = b.inst(Operation::Br { target: hdr }, vec![], None);
        let (i_nd, x) = b.inst(Operation::CallDirect { callee: nd }, vec![], Some(I32));
        let zero = b.konst(0);
        let (i_ne, ne) = b.inst(
            Operation::BinaryOp {
                kind: BinaryOp::ICmpNe,
            },
            vec![x.unwrap(), zero],
            Some(I1),
        );
        let (i_br1, _) = b.inst(
            Operation::CondBr {
                then_target: body,
                else_target: exit,
            },
            vec![ne.unwrap()],
            None,
        );
        let (i_br2, _) = b.inst(Operation::Br { target: hdr }, vec![], None);
        let (i_ret, _) = b.inst(Operation::Ret, vec![], None);
        b.func(
            "main",
            vec![],
            vec![
                blk(e, vec![i_br0]),
                blk(hdr, vec![i_nd, i_ne, i_br1]),
                blk(body, vec![i_br2]),
                blk(exit, vec![i_ret]),
            ],
        );
        assert_eq!(prove(&b.m), ExhaustiveProof::Proven);
    }

    /// An unknown extern is modelled as "returns anything, havocs globals"; the
    /// unconditional error after it is therefore still feasible → abstain with
    /// `error-feasible`, never a proof.
    #[test]
    fn unknown_extern_is_havoc_modelled_not_trusted() {
        let mut b = B::new();
        let err = b.decl("reach_error");
        let ext = b.decl("mystery");
        let e = b.bid();
        let (i_ext, _) = b.inst(Operation::CallDirect { callee: ext }, vec![], None);
        let (i_err, _) = b.inst(Operation::CallDirect { callee: err }, vec![], None);
        let (i_ret, _) = b.inst(Operation::Ret, vec![], None);
        b.func("main", vec![], vec![blk(e, vec![i_ext, i_err, i_ret])]);
        assert_eq!(
            prove(&b.m),
            ExhaustiveProof::Abstain("error-feasible".into())
        );
    }

    /// `__VERIFIER_assume(0); reach_error();` — the assumption kills the path.
    #[test]
    fn false_assumption_ends_the_path() {
        let mut b = B::new();
        let err = b.decl("reach_error");
        let asm = b.decl("__VERIFIER_assume");
        let e = b.bid();
        let zero = b.konst(0);
        let (i_asm, _) = b.inst(Operation::CallDirect { callee: asm }, vec![zero], None);
        let (i_err, _) = b.inst(Operation::CallDirect { callee: err }, vec![], None);
        let (i_ret, _) = b.inst(Operation::Ret, vec![], None);
        b.func("main", vec![], vec![blk(e, vec![i_asm, i_err, i_ret])]);
        assert_eq!(prove(&b.m), ExhaustiveProof::Proven);
    }

    /// A call into a defined helper that asserts an always-true fact:
    /// `check(int c) { if (!c) reach_error(); }  main() { check(7 > 3); }`.
    #[test]
    fn calls_are_inlined_with_bound_arguments() {
        let mut b = B::new();
        let err = b.decl("reach_error");
        let (ce, c_then, c_end) = (b.bid(), b.bid(), b.bid());
        let p = AirParam {
            id: ValueId::new(b.id()),
            name: Some("c".into()),
            index: 0,
            param_type: Some(I32),
        };
        let zero = b.konst(0);
        let (i_eq, is0) = b.inst(
            Operation::BinaryOp {
                kind: BinaryOp::ICmpEq,
            },
            vec![p.id, zero],
            Some(I1),
        );
        let (i_br, _) = b.inst(
            Operation::CondBr {
                then_target: c_then,
                else_target: c_end,
            },
            vec![is0.unwrap()],
            None,
        );
        let (i_err, _) = b.inst(Operation::CallDirect { callee: err }, vec![], None);
        let (i_r1, _) = b.inst(Operation::Ret, vec![], None);
        let (i_r2, _) = b.inst(Operation::Ret, vec![], None);
        let check = b.func(
            "check",
            vec![p],
            vec![
                blk(ce, vec![i_eq, i_br]),
                blk(c_then, vec![i_err, i_r1]),
                blk(c_end, vec![i_r2]),
            ],
        );
        let me = b.bid();
        let (seven, three) = (b.konst(7), b.konst(3));
        let (i_gt, gt) = b.inst(
            Operation::BinaryOp {
                kind: BinaryOp::ICmpSgt,
            },
            vec![seven, three],
            Some(I1),
        );
        let (i_zext, z) = b.inst(
            Operation::Cast {
                kind: CastKind::ZExt,
                target_bits: Some(32),
            },
            vec![gt.unwrap()],
            Some(I32),
        );
        let (i_call, _) = b.inst(
            Operation::CallDirect { callee: check },
            vec![z.unwrap()],
            None,
        );
        let (i_ret, _) = b.inst(Operation::Ret, vec![], None);
        b.func(
            "main",
            vec![],
            vec![blk(me, vec![i_gt, i_zext, i_call, i_ret])],
        );
        assert_eq!(prove(&b.m), ExhaustiveProof::Proven);
    }

    /// `int s; if (s == 7 && s == 8) reach_error();` — `s` is read before any
    /// assignment (an `undef` operand with no definition). Every value is
    /// admitted, and the contradictory guard still proves the error dead.
    #[test]
    fn undefined_operand_is_admitted_as_any_value() {
        let mut b = B::new();
        let err = b.decl("reach_error");
        let (e, bb_then, bb_end) = (b.bid(), b.bid(), b.bid());
        let undef = ValueId::new(b.id()); // no definition anywhere
        let (seven, eight) = (b.konst(7), b.konst(8));
        let (i_eq7, c7) = b.inst(
            Operation::BinaryOp {
                kind: BinaryOp::ICmpEq,
            },
            vec![undef, seven],
            Some(I1),
        );
        let (i_eq8, c8) = b.inst(
            Operation::BinaryOp {
                kind: BinaryOp::ICmpEq,
            },
            vec![undef, eight],
            Some(I1),
        );
        let (i_and, both) = b.inst(
            Operation::BinaryOp {
                kind: BinaryOp::And,
            },
            vec![c7.unwrap(), c8.unwrap()],
            Some(I1),
        );
        let (i_br, _) = b.inst(
            Operation::CondBr {
                then_target: bb_then,
                else_target: bb_end,
            },
            vec![both.unwrap()],
            None,
        );
        let (i_err, _) = b.inst(Operation::CallDirect { callee: err }, vec![], None);
        let (i_r1, _) = b.inst(Operation::Ret, vec![], None);
        let (i_r2, _) = b.inst(Operation::Ret, vec![], None);
        b.func(
            "main",
            vec![],
            vec![
                blk(e, vec![i_eq7, i_eq8, i_and, i_br]),
                blk(bb_then, vec![i_err, i_r1]),
                blk(bb_end, vec![i_r2]),
            ],
        );
        assert_eq!(prove(&b.m), ExhaustiveProof::Proven);
    }

    /// `int a, b; if (&a == &b) reach_error();` — distinct objects never alias.
    #[test]
    fn distinct_allocas_compare_unequal() {
        let mut b = B::new();
        let err = b.decl("reach_error");
        let (e, bb_then, bb_end) = (b.bid(), b.bid(), b.bid());
        let (i_a, pa) = b.inst(
            Operation::Alloca {
                size_bytes: Some(4),
            },
            vec![],
            Some(I32),
        );
        let (i_b, pb) = b.inst(
            Operation::Alloca {
                size_bytes: Some(4),
            },
            vec![],
            Some(I32),
        );
        let (i_eq, c) = b.inst(
            Operation::BinaryOp {
                kind: BinaryOp::ICmpEq,
            },
            vec![pa.unwrap(), pb.unwrap()],
            Some(I1),
        );
        let (i_br, _) = b.inst(
            Operation::CondBr {
                then_target: bb_then,
                else_target: bb_end,
            },
            vec![c.unwrap()],
            None,
        );
        let (i_err, _) = b.inst(Operation::CallDirect { callee: err }, vec![], None);
        let (i_r1, _) = b.inst(Operation::Ret, vec![], None);
        let (i_r2, _) = b.inst(Operation::Ret, vec![], None);
        b.func(
            "main",
            vec![],
            vec![
                blk(e, vec![i_a, i_b, i_eq, i_br]),
                blk(bb_then, vec![i_err, i_r1]),
                blk(bb_end, vec![i_r2]),
            ],
        );
        assert_eq!(prove(&b.m), ExhaustiveProof::Proven);
    }

    /// Twenty independent `if (nondet) x += 1;` steps make 2^20 paths; with
    /// merging at each join the exploration is linear, and `x > 20` is still
    /// proven unreachable (`x <= 20` exactly).
    #[test]
    fn join_point_merging_defeats_path_explosion() {
        let mut b = B::new();
        let err = b.decl("reach_error");
        let nd = b.decl("__VERIFIER_nondet_int");
        let (zero, one, twenty) = (b.konst(0), b.konst(1), b.konst(20));
        let mut blocks = Vec::new();
        let mut cur = b.bid();
        let mut x = zero;
        let n = 20;
        for _ in 0..n {
            let (then_b, join_b) = (b.bid(), b.bid());
            let (i_nd, c) = b.inst(Operation::CallDirect { callee: nd }, vec![], Some(I32));
            let (i_ne, ne) = b.inst(
                Operation::BinaryOp {
                    kind: BinaryOp::ICmpNe,
                },
                vec![c.unwrap(), zero],
                Some(I1),
            );
            let (i_br, _) = b.inst(
                Operation::CondBr {
                    then_target: then_b,
                    else_target: join_b,
                },
                vec![ne.unwrap()],
                None,
            );
            blocks.push(blk(cur, vec![i_nd, i_ne, i_br]));
            let (i_add, x1) = b.inst(
                Operation::BinaryOp {
                    kind: BinaryOp::Add,
                },
                vec![x, one],
                Some(I32),
            );
            let (i_j, _) = b.inst(Operation::Br { target: join_b }, vec![], None);
            blocks.push(blk(then_b, vec![i_add, i_j]));
            let (mut i_phi, xj) = b.inst(Operation::Phi { incoming: vec![] }, vec![], Some(I32));
            i_phi.op = Operation::Phi {
                incoming: vec![(cur, x), (then_b, x1.unwrap())],
            };
            // the join block continues as the next step's head
            let next = b.bid();
            let (i_j2, _) = b.inst(Operation::Br { target: next }, vec![], None);
            blocks.push(blk(join_b, vec![i_phi, i_j2]));
            x = xj.unwrap();
            cur = next;
        }
        let (bb_then, bb_end) = (b.bid(), b.bid());
        let (i_gt, gt) = b.inst(
            Operation::BinaryOp {
                kind: BinaryOp::ICmpSgt,
            },
            vec![x, twenty],
            Some(I1),
        );
        let (i_br, _) = b.inst(
            Operation::CondBr {
                then_target: bb_then,
                else_target: bb_end,
            },
            vec![gt.unwrap()],
            None,
        );
        blocks.push(blk(cur, vec![i_gt, i_br]));
        let (i_err, _) = b.inst(Operation::CallDirect { callee: err }, vec![], None);
        let (i_r1, _) = b.inst(Operation::Ret, vec![], None);
        let (i_r2, _) = b.inst(Operation::Ret, vec![], None);
        blocks.push(blk(bb_then, vec![i_err, i_r1]));
        blocks.push(blk(bb_end, vec![i_r2]));
        b.func("main", vec![], blocks);
        assert_eq!(prove(&b.m), ExhaustiveProof::Proven);
    }

    /// `int g = 0; main() { mystery(); if (g != 0) reach_error(); }` — the extern
    /// may have written `g`, so the guard is feasible: abstain, never a proof.
    #[test]
    fn unknown_extern_havocs_globals() {
        let mut b = B::new();
        let err = b.decl("reach_error");
        let ext = b.decl("mystery");
        let g = ValueId::new(b.id());
        let obj = saf_core::ids::ObjId::new(b.id());
        b.m.globals.push(saf_core::air::AirGlobal {
            id: g,
            obj,
            name: "g".into(),
            init: Some(Constant::Int { value: 0, bits: 32 }),
            is_constant: false,
            span: None,
            value_type: None,
            layout: Some(saf_core::air::Layout::Scalar { size: 4 }),
        });
        let (e, bb_then, bb_end) = (b.bid(), b.bid(), b.bid());
        let (i_ext, _) = b.inst(Operation::CallDirect { callee: ext }, vec![], None);
        let (i_ld, gv) = b.inst(Operation::Load, vec![g], Some(I32));
        let zero = b.konst(0);
        let (i_ne, ne) = b.inst(
            Operation::BinaryOp {
                kind: BinaryOp::ICmpNe,
            },
            vec![gv.unwrap(), zero],
            Some(I1),
        );
        let (i_br, _) = b.inst(
            Operation::CondBr {
                then_target: bb_then,
                else_target: bb_end,
            },
            vec![ne.unwrap()],
            None,
        );
        let (i_err, _) = b.inst(Operation::CallDirect { callee: err }, vec![], None);
        let (i_r1, _) = b.inst(Operation::Ret, vec![], None);
        let (i_r2, _) = b.inst(Operation::Ret, vec![], None);
        b.func(
            "main",
            vec![],
            vec![
                blk(e, vec![i_ext, i_ld, i_ne, i_br]),
                blk(bb_then, vec![i_err, i_r1]),
                blk(bb_end, vec![i_r2]),
            ],
        );
        assert_eq!(
            prove(&b.m),
            ExhaustiveProof::Abstain("error-feasible".into())
        );
    }

    /// The same program WITHOUT the extern call is proven: the initialiser holds.
    #[test]
    fn global_initialiser_is_trusted_without_externs() {
        let mut b = B::new();
        let err = b.decl("reach_error");
        let g = ValueId::new(b.id());
        let obj = saf_core::ids::ObjId::new(b.id());
        b.m.globals.push(saf_core::air::AirGlobal {
            id: g,
            obj,
            name: "g".into(),
            init: Some(Constant::Int { value: 0, bits: 32 }),
            is_constant: false,
            span: None,
            value_type: None,
            layout: Some(saf_core::air::Layout::Scalar { size: 4 }),
        });
        let (e, bb_then, bb_end) = (b.bid(), b.bid(), b.bid());
        let (i_ld, gv) = b.inst(Operation::Load, vec![g], Some(I32));
        let zero = b.konst(0);
        let (i_ne, ne) = b.inst(
            Operation::BinaryOp {
                kind: BinaryOp::ICmpNe,
            },
            vec![gv.unwrap(), zero],
            Some(I1),
        );
        let (i_br, _) = b.inst(
            Operation::CondBr {
                then_target: bb_then,
                else_target: bb_end,
            },
            vec![ne.unwrap()],
            None,
        );
        let (i_err, _) = b.inst(Operation::CallDirect { callee: err }, vec![], None);
        let (i_r1, _) = b.inst(Operation::Ret, vec![], None);
        let (i_r2, _) = b.inst(Operation::Ret, vec![], None);
        b.func(
            "main",
            vec![],
            vec![
                blk(e, vec![i_ld, i_ne, i_br]),
                blk(bb_then, vec![i_err, i_r1]),
                blk(bb_end, vec![i_r2]),
            ],
        );
        assert_eq!(prove(&b.m), ExhaustiveProof::Proven);
    }

    /// `setjmp`/`longjmp` cannot be havoc-modelled (non-local control flow).
    #[test]
    fn setjmp_abstains() {
        let mut b = B::new();
        let sj = b.decl("_setjmp");
        let e = b.bid();
        let (i_sj, _) = b.inst(Operation::CallDirect { callee: sj }, vec![], Some(I32));
        let (i_r, _) = b.inst(Operation::Ret, vec![], None);
        b.func("main", vec![], vec![blk(e, vec![i_sj, i_r])]);
        assert_eq!(
            prove(&b.m),
            ExhaustiveProof::Abstain("extern-_setjmp-nonlocal-control".into())
        );
    }

    /// A global the frontend collapsed a constant pointer expression onto is
    /// opaque: its initialiser must NOT be trusted (the error stays feasible),
    /// while the same program with an exact frontend record is proven.
    #[test]
    fn collapsed_global_is_opaque() {
        for (collapsed, expect) in [
            (true, ExhaustiveProof::Abstain("error-feasible".into())),
            (false, ExhaustiveProof::Proven),
        ] {
            let mut b = B::new();
            let err = b.decl("reach_error");
            let g = ValueId::new(b.id());
            let obj = saf_core::ids::ObjId::new(b.id());
            b.m.globals.push(saf_core::air::AirGlobal {
                id: g,
                obj,
                name: "g".into(),
                init: Some(Constant::Int { value: 0, bits: 32 }),
                is_constant: false,
                span: None,
                value_type: None,
                layout: Some(saf_core::air::Layout::Scalar { size: 4 }),
            });
            b.m.collapsed_globals = Some(if collapsed {
                BTreeSet::from([g])
            } else {
                BTreeSet::new()
            });
            let (e, bb_then, bb_end) = (b.bid(), b.bid(), b.bid());
            let (i_ld, gv) = b.inst(Operation::Load, vec![g], Some(I32));
            let zero = b.konst(0);
            let (i_ne, ne) = b.inst(
                Operation::BinaryOp {
                    kind: BinaryOp::ICmpNe,
                },
                vec![gv.unwrap(), zero],
                Some(I1),
            );
            let (i_br, _) = b.inst(
                Operation::CondBr {
                    then_target: bb_then,
                    else_target: bb_end,
                },
                vec![ne.unwrap()],
                None,
            );
            let (i_err, _) = b.inst(Operation::CallDirect { callee: err }, vec![], None);
            let (i_r1, _) = b.inst(Operation::Ret, vec![], None);
            let (i_r2, _) = b.inst(Operation::Ret, vec![], None);
            b.func(
                "main",
                vec![],
                vec![
                    blk(e, vec![i_ld, i_ne, i_br]),
                    blk(bb_then, vec![i_err, i_r1]),
                    blk(bb_end, vec![i_r2]),
                ],
            );
            assert_eq!(prove(&b.m), expect, "collapsed={collapsed}");
        }
    }

    /// Helper: `main` with one `while(1)` loop whose body is built by `body`:
    /// the loop header has phis for `vars` (initial, from body), body returns
    /// the next values; `err_guard` builds the error condition from the header
    /// phi values. Returns the module.
    fn loop_program(
        b: &mut B,
        inits: &[i64],
        body: impl Fn(&mut B, &[ValueId]) -> (Vec<Instruction>, Vec<ValueId>),
        guard: impl Fn(&mut B, &[ValueId]) -> (Vec<Instruction>, ValueId),
    ) {
        let err = b.decl("reach_error");
        let (e, hdr, chk_then, body_b) = (b.bid(), b.bid(), b.bid(), b.bid());
        let init_ids: Vec<ValueId> = inits.iter().map(|v| b.konst(*v)).collect();
        let phis: Vec<ValueId> = inits.iter().map(|_| ValueId::new(b.id())).collect();
        let (g_insts, cond) = guard(b, &phis);
        let (i_br, _) = b.inst(
            Operation::CondBr {
                then_target: chk_then,
                else_target: body_b,
            },
            vec![cond],
            None,
        );
        let (body_insts, nexts) = body(b, &phis);
        let (i_back, _) = b.inst(Operation::Br { target: hdr }, vec![], None);
        let (i_err, _) = b.inst(Operation::CallDirect { callee: err }, vec![], None);
        let (i_r, _) = b.inst(Operation::Ret, vec![], None);
        let (i_br0, _) = b.inst(Operation::Br { target: hdr }, vec![], None);
        let mut hdr_insts = Vec::new();
        for (j, &p) in phis.iter().enumerate() {
            let (mut i_phi, _) = b.inst(Operation::Phi { incoming: vec![] }, vec![], Some(I32));
            i_phi.dst = Some(p);
            i_phi.op = Operation::Phi {
                incoming: vec![(e, init_ids[j]), (body_b, nexts[j])],
            };
            hdr_insts.push(i_phi);
        }
        hdr_insts.extend(g_insts);
        hdr_insts.push(i_br);
        let mut body_all = body_insts;
        body_all.push(i_back);
        b.func(
            "main",
            vec![],
            vec![
                blk(e, vec![i_br0]),
                blk(hdr, hdr_insts),
                blk(chk_then, vec![i_err, i_r]),
                blk(body_b, body_all),
            ],
        );
    }

    /// `while (1) { x = nondet(); if (x > 10 && x < 5) error; }` — unbounded,
    /// so unwinding hits the cap; k-induction with k = 1 proves it.
    #[test]
    fn k_induction_proves_unbounded_loop_with_contradictory_guard() {
        let mut b = B::new();
        let nd = b.decl("__VERIFIER_nondet_int");
        loop_program(
            &mut b,
            &[0],
            |b, _| {
                let (i_nd, x) = b.inst(Operation::CallDirect { callee: nd }, vec![], Some(I32));
                (vec![i_nd], vec![x.unwrap()])
            },
            |b, phis| {
                let (ten, five) = (b.konst(10), b.konst(5));
                let (i1, gt) = b.inst(
                    Operation::BinaryOp {
                        kind: BinaryOp::ICmpSgt,
                    },
                    vec![phis[0], ten],
                    Some(I1),
                );
                let (i2, lt) = b.inst(
                    Operation::BinaryOp {
                        kind: BinaryOp::ICmpSlt,
                    },
                    vec![phis[0], five],
                    Some(I1),
                );
                let (i3, both) = b.inst(
                    Operation::BinaryOp {
                        kind: BinaryOp::And,
                    },
                    vec![gt.unwrap(), lt.unwrap()],
                    Some(I1),
                );
                (vec![i1, i2, i3], both.unwrap())
            },
        );
        assert_eq!(prove(&b.m), ExhaustiveProof::Proven);
    }

    /// `i = 0; while (1) { if (i > 5) error; i = (i + 1) % 3; }` — not 1-inductive
    /// (an arbitrary `i` can be 100) but 2-inductive (after one iteration
    /// `i ∈ {0,1,2}`).
    #[test]
    fn k_induction_needs_k2_for_modular_counter() {
        let mut b = B::new();
        loop_program(
            &mut b,
            &[0],
            |b, phis| {
                let (one, three) = (b.konst(1), b.konst(3));
                let (i1, a) = b.inst(
                    Operation::BinaryOp {
                        kind: BinaryOp::Add,
                    },
                    vec![phis[0], one],
                    Some(I32),
                );
                let (i2, m) = b.inst(
                    Operation::BinaryOp {
                        kind: BinaryOp::SRem,
                    },
                    vec![a.unwrap(), three],
                    Some(I32),
                );
                (vec![i1, i2], vec![m.unwrap()])
            },
            |b, phis| {
                let five = b.konst(5);
                let (i1, gt) = b.inst(
                    Operation::BinaryOp {
                        kind: BinaryOp::ICmpSgt,
                    },
                    vec![phis[0], five],
                    Some(I1),
                );
                (vec![i1], gt.unwrap())
            },
        );
        assert_eq!(prove(&b.m), ExhaustiveProof::Proven);
    }

    /// `i = 0; while (1) { if (i == 7000) error; i++; }` — the error IS reachable
    /// (at iteration 7001, beyond the unwind cap); k-induction must NOT prove it:
    /// the step case from an arbitrary `i` finds `i == 7000` feasible.
    #[test]
    fn k_induction_does_not_prove_a_reachable_error() {
        let mut b = B::new();
        loop_program(
            &mut b,
            &[0],
            |b, phis| {
                let one = b.konst(1);
                let (i1, a) = b.inst(
                    Operation::BinaryOp {
                        kind: BinaryOp::Add,
                    },
                    vec![phis[0], one],
                    Some(I32),
                );
                (vec![i1], vec![a.unwrap()])
            },
            |b, phis| {
                let c = b.konst(7000);
                let (i1, eq) = b.inst(
                    Operation::BinaryOp {
                        kind: BinaryOp::ICmpEq,
                    },
                    vec![phis[0], c],
                    Some(I1),
                );
                (vec![i1], eq.unwrap())
            },
        );
        match prove(&b.m) {
            ExhaustiveProof::Proven => panic!("wrong proof of a reachable error"),
            ExhaustiveProof::Abstain(r) => assert!(r.contains("k-induction"), "{r}"),
        }
    }

    /// `main() { run(nondet()); }  run(int s) { while (1) { x = nondet(); if (x > 10 && x < 5) error; } }`
    /// — the loop is in a callee of a loop-free `main` (the `s3_srvr` shape);
    /// k-induction slice 2 proves it.
    #[test]
    fn k_induction_handles_a_loop_in_mains_single_looping_callee() {
        let mut b = B::new();
        let err = b.decl("reach_error");
        let nd = b.decl("__VERIFIER_nondet_int");
        let run_id = FunctionId::new(b.id());
        let sp = AirParam {
            id: ValueId::new(b.id()),
            name: Some("s".into()),
            index: 0,
            param_type: Some(I32),
        };
        let (re, hdr, chk, body) = (b.bid(), b.bid(), b.bid(), b.bid());
        let (i_j, _) = b.inst(Operation::Br { target: hdr }, vec![], None);
        let (i_nd, x) = b.inst(Operation::CallDirect { callee: nd }, vec![], Some(I32));
        let (ten, five) = (b.konst(10), b.konst(5));
        let (i1, gt) = b.inst(
            Operation::BinaryOp {
                kind: BinaryOp::ICmpSgt,
            },
            vec![x.unwrap(), ten],
            Some(I1),
        );
        let (i2, lt) = b.inst(
            Operation::BinaryOp {
                kind: BinaryOp::ICmpSlt,
            },
            vec![x.unwrap(), five],
            Some(I1),
        );
        let (i3, both) = b.inst(
            Operation::BinaryOp {
                kind: BinaryOp::And,
            },
            vec![gt.unwrap(), lt.unwrap()],
            Some(I1),
        );
        let (i_br, _) = b.inst(
            Operation::CondBr {
                then_target: chk,
                else_target: body,
            },
            vec![both.unwrap()],
            None,
        );
        let (i_err, _) = b.inst(Operation::CallDirect { callee: err }, vec![], None);
        let (i_r, _) = b.inst(Operation::Ret, vec![], None);
        let (i_back, _) = b.inst(Operation::Br { target: hdr }, vec![], None);
        b.m.functions.push(AirFunction {
            id: run_id,
            name: "run".into(),
            params: vec![sp],
            blocks: vec![
                blk(re, vec![i_j]),
                blk(hdr, vec![i_nd, i1, i2, i3, i_br]),
                blk(chk, vec![i_err, i_r]),
                blk(body, vec![i_back]),
            ],
            entry_block: Some(re),
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        });
        let me = b.bid();
        let (i_s, sv) = b.inst(Operation::CallDirect { callee: nd }, vec![], Some(I32));
        let (i_call, _) = b.inst(
            Operation::CallDirect { callee: run_id },
            vec![sv.unwrap()],
            None,
        );
        let (i_mr, _) = b.inst(Operation::Ret, vec![], None);
        b.func("main", vec![], vec![blk(me, vec![i_s, i_call, i_mr])]);
        assert_eq!(prove(&b.m), ExhaustiveProof::Proven);
    }

    /// Build `int g = 0; main() { while (1) { if (g == 5) reach_error(); g = g + <step>
    /// [% <modulus>]; } }` — loop state in a GLOBAL, so the header has no phis and
    /// the state parks at index 0 (the shape of the 2026-10-09 vacuous proofs).
    fn global_counter_loop(modulus: Option<i64>) -> AirModule {
        let mut b = B::new();
        let err = b.decl("reach_error");
        let g = ValueId::new(b.id());
        let obj = saf_core::ids::ObjId::new(b.id());
        b.m.globals.push(saf_core::air::AirGlobal {
            id: g,
            obj,
            name: "g".into(),
            init: Some(Constant::Int { value: 0, bits: 32 }),
            is_constant: false,
            span: None,
            value_type: None,
            layout: Some(saf_core::air::Layout::Scalar { size: 4 }),
        });
        let (e, hdr, chk, inc) = (b.bid(), b.bid(), b.bid(), b.bid());
        let (i_j, _) = b.inst(Operation::Br { target: hdr }, vec![], None);
        let (i_ld, gv) = b.inst(Operation::Load, vec![g], Some(I32));
        let five = b.konst(5);
        let (i_eq, eq) = b.inst(
            Operation::BinaryOp {
                kind: BinaryOp::ICmpEq,
            },
            vec![gv.unwrap(), five],
            Some(I1),
        );
        let (i_br, _) = b.inst(
            Operation::CondBr {
                then_target: chk,
                else_target: inc,
            },
            vec![eq.unwrap()],
            None,
        );
        let (i_err, _) = b.inst(Operation::CallDirect { callee: err }, vec![], None);
        let (i_ret, _) = b.inst(Operation::Ret, vec![], None);
        let (i_ld2, gv2) = b.inst(Operation::Load, vec![g], Some(I32));
        let one = b.konst(1);
        let (i_add, sum) = b.inst(
            Operation::BinaryOp {
                kind: BinaryOp::Add,
            },
            vec![gv2.unwrap(), one],
            Some(I32),
        );
        let mut inc_insts = vec![i_ld2, i_add];
        let mut newval = sum.unwrap();
        if let Some(m) = modulus {
            let mk = b.konst(m);
            let (i_rem, r) = b.inst(
                Operation::BinaryOp {
                    kind: BinaryOp::SRem,
                },
                vec![newval, mk],
                Some(I32),
            );
            inc_insts.push(i_rem);
            newval = r.unwrap();
        }
        let (i_st, _) = b.inst(Operation::Store, vec![newval, g], None);
        let (i_back, _) = b.inst(Operation::Br { target: hdr }, vec![], None);
        inc_insts.extend([i_st, i_back]);
        b.func(
            "main",
            vec![],
            vec![
                blk(e, vec![i_j]),
                blk(hdr, vec![i_ld, i_eq, i_br]),
                blk(chk, vec![i_err, i_ret]),
                blk(inc, inc_insts),
            ],
        );
        b.m
    }

    /// `int g = 0; while (1) { if (g == error_at) reach_error(); if (nondet()) g = 1 - g; }`
    /// — `g` toggles between 0 and 1 forever. Unrolling never ends and
    /// k-induction fails (an arbitrary header state has `g == error_at`), so
    /// only the header fixpoint — two concrete header states — can decide it.
    fn toggle_loop(error_at: i64) -> AirModule {
        let mut b = B::new();
        let err = b.decl("reach_error");
        let nd = b.decl("__VERIFIER_nondet_int");
        let g = ValueId::new(b.id());
        let obj = saf_core::ids::ObjId::new(b.id());
        b.m.globals.push(saf_core::air::AirGlobal {
            id: g,
            obj,
            name: "g".into(),
            init: Some(Constant::Int { value: 0, bits: 32 }),
            is_constant: false,
            span: None,
            value_type: None,
            layout: Some(saf_core::air::Layout::Scalar { size: 4 }),
        });
        let (e, hdr, chk, step, flip) = (b.bid(), b.bid(), b.bid(), b.bid(), b.bid());
        let (i_j, _) = b.inst(Operation::Br { target: hdr }, vec![], None);
        let (i_ld, gv) = b.inst(Operation::Load, vec![g], Some(I32));
        let target = b.konst(error_at);
        let (i_eq, eq) = b.inst(
            Operation::BinaryOp {
                kind: BinaryOp::ICmpEq,
            },
            vec![gv.unwrap(), target],
            Some(I1),
        );
        let (i_br, _) = b.inst(
            Operation::CondBr {
                then_target: chk,
                else_target: step,
            },
            vec![eq.unwrap()],
            None,
        );
        let (i_err, _) = b.inst(Operation::CallDirect { callee: err }, vec![], None);
        let (i_ret, _) = b.inst(Operation::Ret, vec![], None);
        let (i_nd, x) = b.inst(Operation::CallDirect { callee: nd }, vec![], Some(I32));
        let zero = b.konst(0);
        let (i_ne, ne) = b.inst(
            Operation::BinaryOp {
                kind: BinaryOp::ICmpNe,
            },
            vec![x.unwrap(), zero],
            Some(I1),
        );
        let (i_br2, _) = b.inst(
            Operation::CondBr {
                then_target: flip,
                else_target: hdr,
            },
            vec![ne.unwrap()],
            None,
        );
        let (i_ld2, gv2) = b.inst(Operation::Load, vec![g], Some(I32));
        let one = b.konst(1);
        let (i_sub, nv) = b.inst(
            Operation::BinaryOp {
                kind: BinaryOp::Sub,
            },
            vec![one, gv2.unwrap()],
            Some(I32),
        );
        let (i_st, _) = b.inst(Operation::Store, vec![nv.unwrap(), g], None);
        let (i_back, _) = b.inst(Operation::Br { target: hdr }, vec![], None);
        b.func(
            "main",
            vec![],
            vec![
                blk(e, vec![i_j]),
                blk(hdr, vec![i_ld, i_eq, i_br]),
                blk(chk, vec![i_err, i_ret]),
                blk(step, vec![i_nd, i_ne, i_br2]),
                blk(flip, vec![i_ld2, i_sub, i_st, i_back]),
            ],
        );
        b.m
    }

    fn prove_ovf(m: &AirModule) -> ExhaustiveProof {
        prove_no_overflow_exhaustive(m, Duration::from_secs(30))
    }

    /// `int x = nondet(); [if (x < 100)] y = x + 1;` with the chosen flavour of
    /// `+`: `nsw` (signed, as clang emits) or plain (unsigned). The guard keeps
    /// `x + 1` in range.
    fn increment_program(guarded: bool, nsw: bool) -> AirModule {
        let mut b = B::new();
        let nd = b.decl("__VERIFIER_nondet_int");
        let (e, bb_add, bb_end) = (b.bid(), b.bid(), b.bid());
        let (i_nd, x) = b.inst(Operation::CallDirect { callee: nd }, vec![], Some(I32));
        let hundred = b.konst(100);
        let one = b.konst(1);
        let (i_lt, lt) = b.inst(
            Operation::BinaryOp {
                kind: BinaryOp::ICmpSlt,
            },
            vec![x.unwrap(), hundred],
            Some(I1),
        );
        let guard = if guarded { lt.unwrap() } else { b.konst(1) };
        let (i_br, _) = b.inst(
            Operation::CondBr {
                then_target: bb_add,
                else_target: bb_end,
            },
            vec![guard],
            None,
        );
        let (mut i_add, _) = b.inst(
            Operation::BinaryOp {
                kind: BinaryOp::Add,
            },
            vec![x.unwrap(), one],
            Some(I32),
        );
        if nsw {
            i_add = i_add.with_no_signed_wrap();
        }
        let (i_r1, _) = b.inst(Operation::Ret, vec![], None);
        let (i_r2, _) = b.inst(Operation::Ret, vec![], None);
        b.func(
            "main",
            vec![],
            vec![
                blk(e, vec![i_nd, i_lt, i_br]),
                blk(bb_add, vec![i_add, i_r1]),
                blk(bb_end, vec![i_r2]),
            ],
        );
        b.m
    }

    #[test]
    fn no_overflow_guarded_signed_increment_is_proven() {
        assert_eq!(
            prove_ovf(&increment_program(true, true)),
            ExhaustiveProof::Proven
        );
    }

    #[test]
    fn no_overflow_unguarded_signed_increment_is_feasible() {
        assert_eq!(
            prove_ovf(&increment_program(false, true)),
            ExhaustiveProof::Abstain("error-feasible".into())
        );
    }

    /// Without `nsw` the add is unsigned: wrapping is defined, never an overflow.
    #[test]
    fn no_overflow_unsigned_increment_is_proven() {
        assert_eq!(
            prove_ovf(&increment_program(false, false)),
            ExhaustiveProof::Proven
        );
    }

    /// `reach_error()` is only an abort under `no-overflow`; the unreach goal
    /// on the same program is still a feasible error.
    #[test]
    fn no_overflow_ignores_reach_error_calls() {
        let mut b = B::new();
        let err = b.decl("reach_error");
        let e = b.bid();
        let (i_err, _) = b.inst(Operation::CallDirect { callee: err }, vec![], None);
        let (i_r, _) = b.inst(Operation::Ret, vec![], None);
        b.func("main", vec![], vec![blk(e, vec![i_err, i_r])]);
        assert_eq!(prove_ovf(&b.m), ExhaustiveProof::Proven);
        assert_eq!(
            prove(&b.m),
            ExhaustiveProof::Abstain("error-feasible".into())
        );
    }

    /// `x / y` with both unknown can be `INT_MIN / -1`; with `y > 0` it cannot.
    fn sdiv_program(guarded: bool) -> AirModule {
        let mut b = B::new();
        let nd = b.decl("__VERIFIER_nondet_int");
        let (e, bb_div, bb_end) = (b.bid(), b.bid(), b.bid());
        let (i_x, x) = b.inst(Operation::CallDirect { callee: nd }, vec![], Some(I32));
        let (i_y, y) = b.inst(Operation::CallDirect { callee: nd }, vec![], Some(I32));
        let zero = b.konst(0);
        let (i_gt, gt) = b.inst(
            Operation::BinaryOp {
                kind: BinaryOp::ICmpSgt,
            },
            vec![y.unwrap(), zero],
            Some(I1),
        );
        let guard = if guarded { gt.unwrap() } else { b.konst(1) };
        let (i_br, _) = b.inst(
            Operation::CondBr {
                then_target: bb_div,
                else_target: bb_end,
            },
            vec![guard],
            None,
        );
        let (i_div, _) = b.inst(
            Operation::BinaryOp {
                kind: BinaryOp::SDiv,
            },
            vec![x.unwrap(), y.unwrap()],
            Some(I32),
        );
        let (i_r1, _) = b.inst(Operation::Ret, vec![], None);
        let (i_r2, _) = b.inst(Operation::Ret, vec![], None);
        b.func(
            "main",
            vec![],
            vec![
                blk(e, vec![i_x, i_y, i_gt, i_br]),
                blk(bb_div, vec![i_div, i_r1]),
                blk(bb_end, vec![i_r2]),
            ],
        );
        b.m
    }

    #[test]
    fn no_overflow_signed_division_min_by_minus_one() {
        assert_eq!(
            prove_ovf(&sdiv_program(false)),
            ExhaustiveProof::Abstain("error-feasible".into())
        );
        assert_eq!(prove_ovf(&sdiv_program(true)), ExhaustiveProof::Proven);
    }

    /// `int *p = malloc(4), *q = malloc(4); *p = 5; *q = 7; if (*p != 5) reach_error();`
    /// — two heap objects are distinct cells; the store through `q` does not
    /// disturb `p`, and the load through `p` is exact.
    #[test]
    fn heap_objects_are_distinct_exact_cells() {
        use saf_core::air::HeapAllocKind;
        let mut b = B::new();
        let err = b.decl("reach_error");
        let (e, bb_then, bb_end) = (b.bid(), b.bid(), b.bid());
        let (i_p, p) = b.inst(
            Operation::HeapAlloc {
                kind: HeapAllocKind::Malloc,
            },
            vec![],
            Some(I32),
        );
        let (i_q, q) = b.inst(
            Operation::HeapAlloc {
                kind: HeapAllocKind::Malloc,
            },
            vec![],
            Some(I32),
        );
        let (five, seven) = (b.konst(5), b.konst(7));
        let (i_sp, _) = b.inst(Operation::Store, vec![five, p.unwrap()], None);
        let (i_sq, _) = b.inst(Operation::Store, vec![seven, q.unwrap()], None);
        let (i_ld, v) = b.inst(Operation::Load, vec![p.unwrap()], Some(I32));
        let (i_ne, ne) = b.inst(
            Operation::BinaryOp {
                kind: BinaryOp::ICmpNe,
            },
            vec![v.unwrap(), five],
            Some(I1),
        );
        let (i_br, _) = b.inst(
            Operation::CondBr {
                then_target: bb_then,
                else_target: bb_end,
            },
            vec![ne.unwrap()],
            None,
        );
        let (i_err, _) = b.inst(Operation::CallDirect { callee: err }, vec![], None);
        let (i_r1, _) = b.inst(Operation::Ret, vec![], None);
        let (i_r2, _) = b.inst(Operation::Ret, vec![], None);
        b.func(
            "main",
            vec![],
            vec![
                blk(e, vec![i_p, i_q, i_sp, i_sq, i_ld, i_ne, i_br]),
                blk(bb_then, vec![i_err, i_r1]),
                blk(bb_end, vec![i_r2]),
            ],
        );
        assert_eq!(prove(&b.m), ExhaustiveProof::Proven);
    }

    /// `int *p = malloc(4); if (*p == 3) reach_error();` — indeterminate contents:
    /// the load is fresh and the error feasible, never a proof.
    #[test]
    fn uninitialised_heap_cell_is_any_value() {
        use saf_core::air::HeapAllocKind;
        let mut b = B::new();
        let err = b.decl("reach_error");
        let (e, bb_then, bb_end) = (b.bid(), b.bid(), b.bid());
        let (i_p, p) = b.inst(
            Operation::HeapAlloc {
                kind: HeapAllocKind::Malloc,
            },
            vec![],
            Some(I32),
        );
        let three = b.konst(3);
        let (i_ld, v) = b.inst(Operation::Load, vec![p.unwrap()], Some(I32));
        let (i_eq, eq) = b.inst(
            Operation::BinaryOp {
                kind: BinaryOp::ICmpEq,
            },
            vec![v.unwrap(), three],
            Some(I1),
        );
        let (i_br, _) = b.inst(
            Operation::CondBr {
                then_target: bb_then,
                else_target: bb_end,
            },
            vec![eq.unwrap()],
            None,
        );
        let (i_err, _) = b.inst(Operation::CallDirect { callee: err }, vec![], None);
        let (i_r1, _) = b.inst(Operation::Ret, vec![], None);
        let (i_r2, _) = b.inst(Operation::Ret, vec![], None);
        b.func(
            "main",
            vec![],
            vec![
                blk(e, vec![i_p, i_ld, i_eq, i_br]),
                blk(bb_then, vec![i_err, i_r1]),
                blk(bb_end, vec![i_r2]),
            ],
        );
        assert_eq!(
            prove(&b.m),
            ExhaustiveProof::Abstain("error-feasible".into())
        );
    }

    /// `(long) p == 0` for an alloca `p`: the pointer's integer image is unknown
    /// to the engine, so the comparison admits both outcomes — feasible, never a
    /// proof — and no longer abstains with `cast-PtrToInt`.
    #[test]
    fn ptrtoint_is_an_unknown_integer() {
        let mut b = B::new();
        let err = b.decl("reach_error");
        let (e, bb_then, bb_end) = (b.bid(), b.bid(), b.bid());
        let (i_al, p) = b.inst(
            Operation::Alloca {
                size_bytes: Some(4),
            },
            vec![],
            Some(I32),
        );
        let (i_cast, n) = b.inst(
            Operation::Cast {
                kind: CastKind::PtrToInt,
                target_bits: Some(32),
            },
            vec![p.unwrap()],
            Some(I32),
        );
        let zero = b.konst(0);
        let (i_eq, eq) = b.inst(
            Operation::BinaryOp {
                kind: BinaryOp::ICmpEq,
            },
            vec![n.unwrap(), zero],
            Some(I1),
        );
        let (i_br, _) = b.inst(
            Operation::CondBr {
                then_target: bb_then,
                else_target: bb_end,
            },
            vec![eq.unwrap()],
            None,
        );
        let (i_err, _) = b.inst(Operation::CallDirect { callee: err }, vec![], None);
        let (i_r1, _) = b.inst(Operation::Ret, vec![], None);
        let (i_r2, _) = b.inst(Operation::Ret, vec![], None);
        b.func(
            "main",
            vec![],
            vec![
                blk(e, vec![i_al, i_cast, i_eq, i_br]),
                blk(bb_then, vec![i_err, i_r1]),
                blk(bb_end, vec![i_r2]),
            ],
        );
        assert_eq!(
            prove(&b.m),
            ExhaustiveProof::Abstain("error-feasible".into())
        );
    }

    /// `void use(void *f) {} int main() { use(&use); return 0; }` — a function
    /// used as a value is an opaque pointer; the program is proven.
    #[test]
    fn function_address_operand_is_an_opaque_pointer() {
        let mut b = B::new();
        let (e, ue) = (b.bid(), b.bid());
        let (i_ur, _) = b.inst(Operation::Ret, vec![], None);
        let fp = AirParam {
            id: ValueId::new(b.id()),
            name: Some("f".into()),
            index: 0,
            param_type: Some(PTR),
        };
        let use_fn = b.func("use", vec![fp], vec![blk(ue, vec![i_ur])]);
        let addr = ValueId::new(use_fn.raw());
        let (i_call, _) = b.inst(Operation::CallDirect { callee: use_fn }, vec![addr], None);
        let (i_r, _) = b.inst(Operation::Ret, vec![], None);
        b.func("main", vec![], vec![blk(e, vec![i_call, i_r])]);
        assert_eq!(prove(&b.m), ExhaustiveProof::Proven);
    }

    /// `int a = 0, b = 1; while (1) { if (a == b) reach_error(); t = a; a = b; b = t; }`
    /// — after mem2reg the header phis read each other (`a.0 = phi [b.0]`,
    /// `b.0 = phi [a.0]`). Simultaneous phi semantics keep them distinct
    /// forever (two header states, proven); sequential evaluation made the
    /// swap a copy and reached the error (wrong) — or, with the condition
    /// inverted, hid a real error. Both directions are checked.
    fn swap_loop(error_when_equal: bool) -> AirModule {
        let mut b = B::new();
        let err = b.decl("reach_error");
        let (e, hdr, chk, step) = (b.bid(), b.bid(), b.bid(), b.bid());
        let (i_j, _) = b.inst(Operation::Br { target: hdr }, vec![], None);
        let (mut i_pa, pa) = b.inst(Operation::Phi { incoming: vec![] }, vec![], Some(I32));
        let (mut i_pb, pb) = b.inst(Operation::Phi { incoming: vec![] }, vec![], Some(I32));
        let (zero, one) = (b.konst(0), b.konst(1));
        i_pa.op = Operation::Phi {
            incoming: vec![(e, zero), (step, pb.unwrap())],
        };
        i_pb.op = Operation::Phi {
            incoming: vec![(e, one), (step, pa.unwrap())],
        };
        let (i_cmp, c) = b.inst(
            Operation::BinaryOp {
                kind: if error_when_equal {
                    BinaryOp::ICmpEq
                } else {
                    BinaryOp::ICmpNe
                },
            },
            vec![pa.unwrap(), pb.unwrap()],
            Some(I1),
        );
        let (i_br, _) = b.inst(
            Operation::CondBr {
                then_target: chk,
                else_target: step,
            },
            vec![c.unwrap()],
            None,
        );
        let (i_err, _) = b.inst(Operation::CallDirect { callee: err }, vec![], None);
        let (i_ret, _) = b.inst(Operation::Ret, vec![], None);
        let (i_back, _) = b.inst(Operation::Br { target: hdr }, vec![], None);
        b.func(
            "main",
            vec![],
            vec![
                blk(e, vec![i_j]),
                blk(hdr, vec![i_pa, i_pb, i_cmp, i_br]),
                blk(chk, vec![i_err, i_ret]),
                blk(step, vec![i_back]),
            ],
        );
        b.m
    }

    /// Regression (2026-10-10, `dyn_partition`): phis are simultaneous.
    #[test]
    fn phis_of_a_block_are_evaluated_simultaneously() {
        assert_eq!(prove(&swap_loop(true)), ExhaustiveProof::Proven);
        let r = prove(&swap_loop(false));
        assert!(
            matches!(&r, ExhaustiveProof::Abstain(why) if why.contains("error-feasible")),
            "{r:?}"
        );
    }

    /// `int x = nondet(); int y = nondet(); if (x == 7 && y == 3) reach_error();` —
    /// the counterexample search returns the two reads with their model values.
    #[test]
    fn counterexample_pins_the_nondet_reads() {
        let mut b = B::new();
        let err = b.decl("reach_error");
        let nd = b.decl("__VERIFIER_nondet_int");
        let (e, bb_mid, bb_then, bb_end) = (b.bid(), b.bid(), b.bid(), b.bid());
        let (i_x, x) = b.inst(Operation::CallDirect { callee: nd }, vec![], Some(I32));
        let (i_y, y) = b.inst(Operation::CallDirect { callee: nd }, vec![], Some(I32));
        let (seven, three) = (b.konst(7), b.konst(3));
        let (i_c1, c1) = b.inst(
            Operation::BinaryOp {
                kind: BinaryOp::ICmpEq,
            },
            vec![x.unwrap(), seven],
            Some(I1),
        );
        let (i_b1, _) = b.inst(
            Operation::CondBr {
                then_target: bb_mid,
                else_target: bb_end,
            },
            vec![c1.unwrap()],
            None,
        );
        let (i_c2, c2) = b.inst(
            Operation::BinaryOp {
                kind: BinaryOp::ICmpEq,
            },
            vec![y.unwrap(), three],
            Some(I1),
        );
        let (i_b2, _) = b.inst(
            Operation::CondBr {
                then_target: bb_then,
                else_target: bb_end,
            },
            vec![c2.unwrap()],
            None,
        );
        let (i_err, _) = b.inst(Operation::CallDirect { callee: err }, vec![], None);
        let (i_r1, _) = b.inst(Operation::Ret, vec![], None);
        let (i_r2, _) = b.inst(Operation::Ret, vec![], None);
        let err_id = i_err.id;
        b.func(
            "main",
            vec![],
            vec![
                blk(e, vec![i_x, i_y, i_c1, i_b1]),
                blk(bb_mid, vec![i_c2, i_b2]),
                blk(bb_then, vec![i_err, i_r1]),
                blk(bb_end, vec![i_r2]),
            ],
        );
        let cex =
            find_counterexample(&b.m, Duration::from_secs(30), false).expect("a counterexample");
        assert_eq!(cex.error_inst, err_id);
        let vals: Vec<(String, i64)> = cex
            .nondets
            .iter()
            .map(|(n, v, _)| (n.clone(), *v))
            .collect();
        assert_eq!(
            vals,
            vec![
                ("__VERIFIER_nondet_int".to_string(), 7),
                ("__VERIFIER_nondet_int".to_string(), 3)
            ]
        );
        assert!(cex.block_path.contains(&bb_then));
        // A safe program yields none.
        assert!(find_counterexample(&toggle_loop(5), Duration::from_secs(10), false).is_none());
    }

    /// The toggle never reaches 5: proven by the header fixpoint (2 states).
    #[test]
    fn header_fixpoint_proves_finite_state_unbounded_loop() {
        assert_eq!(prove(&toggle_loop(5)), ExhaustiveProof::Proven);
    }

    /// The proof exports the header's exact state set: `g ∈ {0, 1}`.
    #[test]
    fn header_fixpoint_exports_the_state_set() {
        let m = toggle_loop(5);
        let (proof, states) = prove_unreach_exhaustive_with_states(&m, Duration::from_secs(30));
        assert_eq!(proof, ExhaustiveProof::Proven);
        assert_eq!(states.len(), 1, "{states:?}");
        // `g == 0` appears twice (initialiser on the entry path, then written):
        // the same C state, two footprint shapes.
        let gs: BTreeSet<u64> = states[0]
            .states
            .iter()
            .map(|st| {
                let g: Vec<u64> = st
                    .iter()
                    .filter(|(v, _, _)| matches!(v, StateVar::Global(_)))
                    .map(|(_, val, _)| *val)
                    .collect();
                assert_eq!(g.len(), 1, "{st:?}");
                g[0]
            })
            .collect();
        assert_eq!(gs, BTreeSet::from([0, 1]));
    }

    /// The toggle reaches 1 after one flip: a feasible error, never a proof.
    #[test]
    fn header_fixpoint_does_not_prove_reachable_state() {
        let r = prove(&toggle_loop(1));
        assert!(
            matches!(&r, ExhaustiveProof::Abstain(why) if why.contains("error-feasible")),
            "{r:?}"
        );
    }

    /// `g` counts up without bound and the error needs `g == -1` (2^31
    /// iterations away): the fixpoint's state set overflows its cap, the loop
    /// is unrolled as before, and the result is an abstain — not a proof.
    #[test]
    fn header_fixpoint_gives_up_on_an_unbounded_counter() {
        let mut b = B::new();
        let err = b.decl("reach_error");
        let g = ValueId::new(b.id());
        let obj = saf_core::ids::ObjId::new(b.id());
        b.m.globals.push(saf_core::air::AirGlobal {
            id: g,
            obj,
            name: "g".into(),
            init: Some(Constant::Int { value: 0, bits: 32 }),
            is_constant: false,
            span: None,
            value_type: None,
            layout: Some(saf_core::air::Layout::Scalar { size: 4 }),
        });
        let (e, hdr, chk, inc) = (b.bid(), b.bid(), b.bid(), b.bid());
        let (i_j, _) = b.inst(Operation::Br { target: hdr }, vec![], None);
        let (i_ld, gv) = b.inst(Operation::Load, vec![g], Some(I32));
        let minus_one = b.konst(-1);
        let (i_eq, eq) = b.inst(
            Operation::BinaryOp {
                kind: BinaryOp::ICmpEq,
            },
            vec![gv.unwrap(), minus_one],
            Some(I1),
        );
        let (i_br, _) = b.inst(
            Operation::CondBr {
                then_target: chk,
                else_target: inc,
            },
            vec![eq.unwrap()],
            None,
        );
        let (i_err, _) = b.inst(Operation::CallDirect { callee: err }, vec![], None);
        let (i_ret, _) = b.inst(Operation::Ret, vec![], None);
        let (i_ld2, gv2) = b.inst(Operation::Load, vec![g], Some(I32));
        let one = b.konst(1);
        let (i_add, sum) = b.inst(
            Operation::BinaryOp {
                kind: BinaryOp::Add,
            },
            vec![gv2.unwrap(), one],
            Some(I32),
        );
        let (i_st, _) = b.inst(Operation::Store, vec![sum.unwrap(), g], None);
        let (i_back, _) = b.inst(Operation::Br { target: hdr }, vec![], None);
        b.func(
            "main",
            vec![],
            vec![
                blk(e, vec![i_j]),
                blk(hdr, vec![i_ld, i_eq, i_br]),
                blk(chk, vec![i_err, i_ret]),
                blk(inc, vec![i_ld2, i_add, i_st, i_back]),
            ],
        );
        let r = prove_unreach_exhaustive(&b.m, Duration::from_secs(10));
        assert!(!matches!(r, ExhaustiveProof::Proven), "{r:?}");
    }

    /// Regression (2026-10-09): the error at iteration 6 of a global counter was
    /// "proven" because a phi-less header's entry was counted twice and the step
    /// case ended before its checked iteration ran.
    #[test]
    fn k_induction_does_not_prove_reachable_error_with_global_state() {
        let m = global_counter_loop(None);
        let r = prove(&m);
        assert!(
            matches!(&r, ExhaustiveProof::Abstain(why) if why.contains("error-feasible")),
            "{r:?}"
        );
    }

    /// `g = (g + 1) % 3` leaves `g` in -2..=2 whatever it was before, so after
    /// ANY iteration `g == 5` is unreachable: 1-inductive, and proven — with the
    /// same phi-less global-state shape as the regression above.
    #[test]
    fn k_induction_proves_global_modular_counter() {
        let m = global_counter_loop(Some(3));
        assert_eq!(prove(&m), ExhaustiveProof::Proven);
    }

    /// `struct { int a; int b; } s; s.b = 5; s.a = 1; if (s.b != 5) reach_error();`
    /// — two constant-path cells under one alloca; the field write to `a` must
    /// not disturb `b`, and the load of `b` is exact.
    #[test]
    fn struct_fields_are_distinct_exact_cells() {
        use saf_core::air::{FieldPath, FieldStep};
        let mut b = B::new();
        let err = b.decl("reach_error");
        let (e, bb_then, bb_end) = (b.bid(), b.bid(), b.bid());
        let (i_al, s) = b.inst(
            Operation::Alloca {
                size_bytes: Some(8),
            },
            vec![],
            Some(I32),
        );
        let (zero, one, five) = (b.konst(0), b.konst(1), b.konst(5));
        let path = |ix: u32| FieldPath {
            steps: vec![
                FieldStep::Field { index: 0 },
                FieldStep::Field { index: ix },
            ],
        };
        let (mut i_gb, pb) = b.inst(
            Operation::Gep {
                field_path: path(1),
            },
            vec![s.unwrap(), zero, one],
            Some(I32),
        );
        i_gb.span = Some(point_span(3, 5));
        gep_bytes(&mut i_gb, &[(false, 8), (true, 4)]);
        let (i_sb, _) = b.inst(Operation::Store, vec![five, pb.unwrap()], None);
        let (mut i_ga, pa) = b.inst(
            Operation::Gep {
                field_path: path(0),
            },
            vec![s.unwrap(), zero, zero],
            Some(I32),
        );
        i_ga.span = Some(point_span(4, 5));
        gep_bytes(&mut i_ga, &[(false, 8), (true, 0)]);
        let (i_sa, _) = b.inst(Operation::Store, vec![one, pa.unwrap()], None);
        let (i_ld, v) = b.inst(Operation::Load, vec![pb.unwrap()], Some(I32));
        let (i_ne, ne) = b.inst(
            Operation::BinaryOp {
                kind: BinaryOp::ICmpNe,
            },
            vec![v.unwrap(), five],
            Some(I1),
        );
        let (i_br, _) = b.inst(
            Operation::CondBr {
                then_target: bb_then,
                else_target: bb_end,
            },
            vec![ne.unwrap()],
            None,
        );
        let (i_err, _) = b.inst(Operation::CallDirect { callee: err }, vec![], None);
        let (i_r1, _) = b.inst(Operation::Ret, vec![], None);
        let (i_r2, _) = b.inst(Operation::Ret, vec![], None);
        b.func(
            "main",
            vec![],
            vec![
                blk(e, vec![i_al, i_gb, i_sb, i_ga, i_sa, i_ld, i_ne, i_br]),
                blk(bb_then, vec![i_err, i_r1]),
                blk(bb_end, vec![i_r2]),
            ],
        );
        assert_eq!(prove(&b.m), ExhaustiveProof::Proven);
    }

    /// `a[0] = 7; a[i] = 1; if (a[0] != 7) reach_error();` with a nondet `i`:
    /// the store at an unknown offset forgets every cell of `a`, so the later
    /// load is fresh and the error is feasible — never a guess that `a[0]`
    /// survived (`i` may be 0).
    #[test]
    fn dynamic_index_store_havocs_the_object() {
        use saf_core::air::{FieldPath, FieldStep};
        let mut b = B::new();
        let err = b.decl("reach_error");
        let nd = b.decl("__VERIFIER_nondet_int");
        let (e, bb_then, bb_end) = (b.bid(), b.bid(), b.bid());
        let (i_al, a) = b.inst(
            Operation::Alloca {
                size_bytes: Some(40),
            },
            vec![],
            Some(I32),
        );
        let (i_nd, i) = b.inst(Operation::CallDirect { callee: nd }, vec![], Some(I32));
        let (zero, one, seven) = (b.konst(0), b.konst(1), b.konst(7));
        let (mut i_g0, p0) = b.inst(
            Operation::Gep {
                field_path: FieldPath {
                    steps: vec![FieldStep::Field { index: 0 }, FieldStep::Field { index: 0 }],
                },
            },
            vec![a.unwrap(), zero, zero],
            Some(I32),
        );
        i_g0.span = Some(point_span(4, 5));
        gep_bytes(&mut i_g0, &[(false, 40), (false, 4)]);
        let (i_st0, _) = b.inst(Operation::Store, vec![seven, p0.unwrap()], None);
        let (mut i_g, p) = b.inst(
            Operation::Gep {
                field_path: FieldPath {
                    steps: vec![FieldStep::Field { index: 0 }, FieldStep::Index],
                },
            },
            vec![a.unwrap(), zero, i.unwrap()],
            Some(I32),
        );
        i_g.span = Some(point_span(5, 5));
        gep_bytes(&mut i_g, &[(false, 40), (false, 4)]);
        let (i_st, _) = b.inst(Operation::Store, vec![one, p.unwrap()], None);
        let (i_ld, v) = b.inst(Operation::Load, vec![p0.unwrap()], Some(I32));
        let (i_ne, ne) = b.inst(
            Operation::BinaryOp {
                kind: BinaryOp::ICmpNe,
            },
            vec![v.unwrap(), seven],
            Some(I1),
        );
        let (i_br, _) = b.inst(
            Operation::CondBr {
                then_target: bb_then,
                else_target: bb_end,
            },
            vec![ne.unwrap()],
            None,
        );
        let (i_err, _) = b.inst(Operation::CallDirect { callee: err }, vec![], None);
        let (i_r1, _) = b.inst(Operation::Ret, vec![], None);
        let (i_r2, _) = b.inst(Operation::Ret, vec![], None);
        b.func(
            "main",
            vec![],
            vec![
                blk(
                    e,
                    vec![i_al, i_nd, i_g0, i_st0, i_g, i_st, i_ld, i_ne, i_br],
                ),
                blk(bb_then, vec![i_err, i_r1]),
                blk(bb_end, vec![i_r2]),
            ],
        );
        assert_eq!(
            prove(&b.m),
            ExhaustiveProof::Abstain("error-feasible".into())
        );
    }

    /// `int g[2] = {3, 4}; if (g[1] != 4) reach_error();` — the constant GEP on
    /// the global is the frontend's synthetic form (no span, first index already
    /// dropped); the load reads the initialiser at path `[1]`.
    #[test]
    fn global_aggregate_initialiser_is_read_by_path() {
        use saf_core::air::{AirGlobal, FieldPath, FieldStep};
        use saf_core::ids::ObjId;
        let mut b = B::new();
        let err = b.decl("reach_error");
        let gid = ValueId::new(b.id());
        let mut g = AirGlobal::new(gid, ObjId::new(b.id()), "g");
        g.init = Some(Constant::Aggregate {
            elements: vec![
                Constant::Int { value: 3, bits: 32 },
                Constant::Int { value: 4, bits: 32 },
            ],
        });
        g.layout = Some(saf_core::air::Layout::Array {
            count: 2,
            stride: 4,
            element: Box::new(saf_core::air::Layout::Scalar { size: 4 }),
        });
        b.m.globals.push(g);
        let (e, bb_then, bb_end) = (b.bid(), b.bid(), b.bid());
        let one = b.konst(1);
        let four = b.konst(4);
        let (mut i_g, p) = b.inst(
            Operation::Gep {
                field_path: FieldPath {
                    steps: vec![FieldStep::Field { index: 1 }],
                },
            },
            vec![gid, one],
            Some(I32),
        );
        gep_bytes(&mut i_g, &[(true, 4)]);
        let (i_ld, v) = b.inst(Operation::Load, vec![p.unwrap()], Some(I32));
        let (i_ne, ne) = b.inst(
            Operation::BinaryOp {
                kind: BinaryOp::ICmpNe,
            },
            vec![v.unwrap(), four],
            Some(I1),
        );
        let (i_br, _) = b.inst(
            Operation::CondBr {
                then_target: bb_then,
                else_target: bb_end,
            },
            vec![ne.unwrap()],
            None,
        );
        let (i_err, _) = b.inst(Operation::CallDirect { callee: err }, vec![], None);
        let (i_r1, _) = b.inst(Operation::Ret, vec![], None);
        let (i_r2, _) = b.inst(Operation::Ret, vec![], None);
        b.func(
            "main",
            vec![],
            vec![
                blk(e, vec![i_g, i_ld, i_ne, i_br]),
                blk(bb_then, vec![i_err, i_r1]),
                blk(bb_end, vec![i_r2]),
            ],
        );
        assert_eq!(prove(&b.m), ExhaustiveProof::Proven);
    }

    /// `%n` writes through its argument; every other conversion only reads.
    #[test]
    fn printf_format_writes_only_with_percent_n() {
        assert!(!format_writes("%s\n"));
        assert!(!format_writes("%d%% done %ld"));
        assert!(format_writes("%n"));
        assert!(format_writes("x=%d%hhn"));
        assert!(format_writes("%-08.3ln"));
        assert!(!format_writes("100%"));
    }

    /// Regression (2026-10-11, the wrong TRUE on `neural-networks/gcas_6_unsafe`):
    /// `static const int W[3][2] = {{1,2},{3,4},{5,6}}; int (*p)[2] = W;
    /// if (p[0][1] == 2) reach_error();` — the GEP `[2 x i32], ptr %p, 0, 1`
    /// is shaped by its own source type, not the global's `[3 x [2 x i32]]`;
    /// its structural path `[1]` read `W[1][0]` (3) and "proved" the reachable
    /// error away. Cells are byte offsets: `p + 4` is `W[0][1]`, and the
    /// error is feasible (`v == 2`); the mirror `v == 3` is unreachable.
    #[test]
    fn decayed_row_pointer_addresses_by_byte_offset() {
        use saf_core::air::{AirGlobal, FieldPath, FieldStep, Layout};
        use saf_core::ids::ObjId;
        let row = |a: i64, b: i64| Constant::Aggregate {
            elements: vec![
                Constant::Int { value: a, bits: 32 },
                Constant::Int { value: b, bits: 32 },
            ],
        };
        for (expect, proven) in [(2, false), (3, true)] {
            let mut b = B::new();
            let err = b.decl("reach_error");
            let gid = ValueId::new(b.id());
            let mut g = AirGlobal::new(gid, ObjId::new(b.id()), "W");
            g.is_constant = true;
            g.init = Some(Constant::Aggregate {
                elements: vec![row(1, 2), row(3, 4), row(5, 6)],
            });
            g.layout = Some(Layout::Array {
                count: 3,
                stride: 8,
                element: Box::new(Layout::Array {
                    count: 2,
                    stride: 4,
                    element: Box::new(Layout::Scalar { size: 4 }),
                }),
            });
            b.m.globals.push(g);
            let (e, bb_then, bb_end) = (b.bid(), b.bid(), b.bid());
            let (zero, one, want) = (b.konst(0), b.konst(1), b.konst(expect));
            // `getelementptr [2 x i32], ptr @W, i32 0, i32 1` as clang emits for
            // `p[0][1]` through the decayed row pointer.
            let (mut i_g, p) = b.inst(
                Operation::Gep {
                    field_path: FieldPath {
                        steps: vec![FieldStep::Field { index: 0 }, FieldStep::Field { index: 1 }],
                    },
                },
                vec![gid, zero, one],
                Some(I32),
            );
            gep_bytes(&mut i_g, &[(false, 8), (false, 4)]);
            let (i_ld, v) = b.inst(Operation::Load, vec![p.unwrap()], Some(I32));
            let (i_eq, eq) = b.inst(
                Operation::BinaryOp {
                    kind: BinaryOp::ICmpEq,
                },
                vec![v.unwrap(), want],
                Some(I1),
            );
            let (i_br, _) = b.inst(
                Operation::CondBr {
                    then_target: bb_then,
                    else_target: bb_end,
                },
                vec![eq.unwrap()],
                None,
            );
            let (i_err, _) = b.inst(Operation::CallDirect { callee: err }, vec![], None);
            let (i_r1, _) = b.inst(Operation::Ret, vec![], None);
            let (i_r2, _) = b.inst(Operation::Ret, vec![], None);
            b.func(
                "main",
                vec![],
                vec![
                    blk(e, vec![i_g, i_ld, i_eq, i_br]),
                    blk(bb_then, vec![i_err, i_r1]),
                    blk(bb_end, vec![i_r2]),
                ],
            );
            let r = prove(&b.m);
            if proven {
                assert_eq!(r, ExhaustiveProof::Proven, "v == {expect}");
            } else {
                assert!(
                    matches!(&r, ExhaustiveProof::Abstain(why) if why.contains("error-feasible")),
                    "v == {expect}: {r:?}"
                );
            }
        }
    }

    /// `short s[4] = {1,2,3,4}; ((int *)s)[0] = 7; if (s[1] == 2) reach_error();`
    /// — a 4-byte store through another view overlaps the 2-byte cell `s[1]`:
    /// it no longer holds its initialiser, and the error is feasible.
    #[test]
    fn overlapping_store_forgets_the_narrower_cell() {
        use saf_core::air::{AirGlobal, FieldPath, FieldStep, Layout};
        use saf_core::ids::ObjId;
        let mut b = B::new();
        let err = b.decl("reach_error");
        let gid = ValueId::new(b.id());
        let mut g = AirGlobal::new(gid, ObjId::new(b.id()), "s");
        g.init = Some(Constant::Aggregate {
            elements: (1..=4)
                .map(|v| Constant::Int { value: v, bits: 16 })
                .collect(),
        });
        g.layout = Some(Layout::Array {
            count: 4,
            stride: 2,
            element: Box::new(Layout::Scalar { size: 2 }),
        });
        b.m.globals.push(g);
        let (e, bb_then, bb_end) = (b.bid(), b.bid(), b.bid());
        let (zero, one, seven) = (b.konst(0), b.konst(1), b.konst(7));
        let (i_st, _) = b.inst(Operation::Store, vec![seven, gid], None);
        let (mut i_g, p) = b.inst(
            Operation::Gep {
                field_path: FieldPath {
                    steps: vec![FieldStep::Field { index: 0 }, FieldStep::Field { index: 1 }],
                },
            },
            vec![gid, zero, one],
            Some(I32),
        );
        gep_bytes(&mut i_g, &[(false, 8), (false, 2)]);
        const I16: TypeId = TypeId::new(16);
        b.m.types.insert(I16, AirType::Integer { bits: 16 });
        let (i_ld, v) = b.inst(Operation::Load, vec![p.unwrap()], Some(I16));
        let two16 = ValueId::new(b.id());
        b.m.constants
            .insert(two16, Constant::Int { value: 2, bits: 16 });
        let (i_eq, eq) = b.inst(
            Operation::BinaryOp {
                kind: BinaryOp::ICmpEq,
            },
            vec![v.unwrap(), two16],
            Some(I1),
        );
        let (i_br, _) = b.inst(
            Operation::CondBr {
                then_target: bb_then,
                else_target: bb_end,
            },
            vec![eq.unwrap()],
            None,
        );
        let (i_err, _) = b.inst(Operation::CallDirect { callee: err }, vec![], None);
        let (i_r1, _) = b.inst(Operation::Ret, vec![], None);
        let (i_r2, _) = b.inst(Operation::Ret, vec![], None);
        b.func(
            "main",
            vec![],
            vec![
                blk(e, vec![i_st, i_g, i_ld, i_eq, i_br]),
                blk(bb_then, vec![i_err, i_r1]),
                blk(bb_end, vec![i_r2]),
            ],
        );
        let r = prove(&b.m);
        assert!(
            matches!(&r, ExhaustiveProof::Abstain(why) if why.contains("error-feasible")),
            "{r:?}"
        );
    }

    /// `int g = 0; if (nondet()) g = 1; if (g > 1) reach_error();` — the global
    /// is written on one branch only; at the join the unwritten side must read
    /// as the initialiser (0), so `g` is `ite(c, 1, 0)` and `g > 1` is dead.
    #[test]
    fn global_written_on_one_branch_merges_against_its_initialiser() {
        use saf_core::air::AirGlobal;
        use saf_core::ids::ObjId;
        let mut b = B::new();
        let err = b.decl("reach_error");
        let nd = b.decl("__VERIFIER_nondet_int");
        let gid = ValueId::new(b.id());
        let mut g = AirGlobal::new(gid, ObjId::new(b.id()), "g");
        g.init = Some(Constant::Int { value: 0, bits: 32 });
        g.layout = Some(saf_core::air::Layout::Scalar { size: 4 });
        b.m.globals.push(g);
        let (e, set_b, join, bb_then, bb_end) = (b.bid(), b.bid(), b.bid(), b.bid(), b.bid());
        let (zero, one) = (b.konst(0), b.konst(1));
        let (i_nd, c) = b.inst(Operation::CallDirect { callee: nd }, vec![], Some(I32));
        let (i_ne, ne) = b.inst(
            Operation::BinaryOp {
                kind: BinaryOp::ICmpNe,
            },
            vec![c.unwrap(), zero],
            Some(I1),
        );
        let (i_br, _) = b.inst(
            Operation::CondBr {
                then_target: set_b,
                else_target: join,
            },
            vec![ne.unwrap()],
            None,
        );
        let (i_st, _) = b.inst(Operation::Store, vec![one, gid], None);
        let (i_j, _) = b.inst(Operation::Br { target: join }, vec![], None);
        let (i_ld, v) = b.inst(Operation::Load, vec![gid], Some(I32));
        let (i_gt, gt) = b.inst(
            Operation::BinaryOp {
                kind: BinaryOp::ICmpSgt,
            },
            vec![v.unwrap(), one],
            Some(I1),
        );
        let (i_br2, _) = b.inst(
            Operation::CondBr {
                then_target: bb_then,
                else_target: bb_end,
            },
            vec![gt.unwrap()],
            None,
        );
        let (i_err, _) = b.inst(Operation::CallDirect { callee: err }, vec![], None);
        let (i_r1, _) = b.inst(Operation::Ret, vec![], None);
        let (i_r2, _) = b.inst(Operation::Ret, vec![], None);
        b.func(
            "main",
            vec![],
            vec![
                blk(e, vec![i_nd, i_ne, i_br]),
                blk(set_b, vec![i_st, i_j]),
                blk(join, vec![i_ld, i_gt, i_br2]),
                blk(bb_then, vec![i_err, i_r1]),
                blk(bb_end, vec![i_r2]),
            ],
        );
        assert_eq!(prove(&b.m), ExhaustiveProof::Proven);
    }

    #[test]
    fn recursion_abstains() {
        let mut b = B::new();
        let fe = b.bid();
        let f_id = FunctionId::new(b.id());
        let (i_self, _) = b.inst(Operation::CallDirect { callee: f_id }, vec![], None);
        let (i_ret, _) = b.inst(Operation::Ret, vec![], None);
        b.m.functions.push(AirFunction {
            id: f_id,
            name: "f".into(),
            params: vec![],
            blocks: vec![blk(fe, vec![i_self, i_ret])],
            entry_block: Some(fe),
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        });
        let me = b.bid();
        let (i_call, _) = b.inst(Operation::CallDirect { callee: f_id }, vec![], None);
        let (i_r, _) = b.inst(Operation::Ret, vec![], None);
        b.func("main", vec![], vec![blk(me, vec![i_call, i_r])]);
        assert_eq!(prove(&b.m), ExhaustiveProof::Abstain("call-depth".into()));
    }

    /// `int id(int x) { if (x == 0) return 0; int r = id(x - 1) + 1; if (r > 5) return 5; return r; }`
    /// `main() { if (id(3) == 10) reach_error(); }` — bounded recursion unwinds to
    /// completion and `id(3) == 3`, so the error is proven dead (the outer
    /// activation's `x` and `r` survive the inner call).
    #[test]
    fn bounded_recursion_is_unwound_with_activation_values_preserved() {
        let mut b = B::new();
        let err = b.decl("reach_error");
        let id_fn = FunctionId::new(b.id());
        let x = AirParam {
            id: ValueId::new(b.id()),
            name: Some("x".into()),
            index: 0,
            param_type: Some(I32),
        };
        let (e, base, rec, big, small) = (b.bid(), b.bid(), b.bid(), b.bid(), b.bid());
        let (zero, one, five) = (b.konst(0), b.konst(1), b.konst(5));
        let (i_eq0, is0) = b.inst(
            Operation::BinaryOp {
                kind: BinaryOp::ICmpEq,
            },
            vec![x.id, zero],
            Some(I1),
        );
        let (i_br0, _) = b.inst(
            Operation::CondBr {
                then_target: base,
                else_target: rec,
            },
            vec![is0.unwrap()],
            None,
        );
        let (i_ret0, _) = b.inst(Operation::Ret, vec![zero], None);
        let (i_sub, xm1) = b.inst(
            Operation::BinaryOp {
                kind: BinaryOp::Sub,
            },
            vec![x.id, one],
            Some(I32),
        );
        let (i_call, r0) = b.inst(
            Operation::CallDirect { callee: id_fn },
            vec![xm1.unwrap()],
            Some(I32),
        );
        let (i_add, r) = b.inst(
            Operation::BinaryOp {
                kind: BinaryOp::Add,
            },
            vec![r0.unwrap(), one],
            Some(I32),
        );
        let (i_gt, gt) = b.inst(
            Operation::BinaryOp {
                kind: BinaryOp::ICmpSgt,
            },
            vec![r.unwrap(), five],
            Some(I1),
        );
        let (i_br1, _) = b.inst(
            Operation::CondBr {
                then_target: big,
                else_target: small,
            },
            vec![gt.unwrap()],
            None,
        );
        let (i_ret5, _) = b.inst(Operation::Ret, vec![five], None);
        let (i_retr, _) = b.inst(Operation::Ret, vec![r.unwrap()], None);
        b.m.functions.push(AirFunction {
            id: id_fn,
            name: "id".into(),
            params: vec![x],
            blocks: vec![
                blk(e, vec![i_eq0, i_br0]),
                blk(base, vec![i_ret0]),
                blk(rec, vec![i_sub, i_call, i_add, i_gt, i_br1]),
                blk(big, vec![i_ret5]),
                blk(small, vec![i_retr]),
            ],
            entry_block: Some(e),
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        });
        let (me, m_then, m_end) = (b.bid(), b.bid(), b.bid());
        let (three, ten) = (b.konst(3), b.konst(10));
        let (i_c, res) = b.inst(
            Operation::CallDirect { callee: id_fn },
            vec![three],
            Some(I32),
        );
        let (i_eq, eq10) = b.inst(
            Operation::BinaryOp {
                kind: BinaryOp::ICmpEq,
            },
            vec![res.unwrap(), ten],
            Some(I1),
        );
        let (i_br, _) = b.inst(
            Operation::CondBr {
                then_target: m_then,
                else_target: m_end,
            },
            vec![eq10.unwrap()],
            None,
        );
        let (i_err, _) = b.inst(Operation::CallDirect { callee: err }, vec![], None);
        let (i_r1, _) = b.inst(Operation::Ret, vec![], None);
        let (i_r2, _) = b.inst(Operation::Ret, vec![], None);
        b.func(
            "main",
            vec![],
            vec![
                blk(me, vec![i_c, i_eq, i_br]),
                blk(m_then, vec![i_err, i_r1]),
                blk(m_end, vec![i_r2]),
            ],
        );
        assert_eq!(prove(&b.m), ExhaustiveProof::Proven);
    }

    /// `y = x + 1; z = y + 1` (both `nsw`, `x` nondet): the search reports the
    /// first overflow; excluding it, the second — the path goes on past a
    /// candidate that did not replay.
    #[test]
    fn counterexample_passes_over_excluded_violations() {
        let mut b = B::new();
        let nd = b.decl("__VERIFIER_nondet_int");
        let e = b.bid();
        let (i_nd, x) = b.inst(Operation::CallDirect { callee: nd }, vec![], Some(I32));
        let one = b.konst(1);
        let (mut i_a, a) = b.inst(
            Operation::BinaryOp {
                kind: BinaryOp::Add,
            },
            vec![x.unwrap(), one],
            Some(I32),
        );
        i_a = i_a.with_no_signed_wrap();
        let (mut i_b, _) = b.inst(
            Operation::BinaryOp {
                kind: BinaryOp::Add,
            },
            vec![a.unwrap(), one],
            Some(I32),
        );
        i_b = i_b.with_no_signed_wrap();
        let (i_r, _) = b.inst(Operation::Ret, vec![], None);
        let (ia, ib) = (i_a.id, i_b.id);
        b.func("main", vec![], vec![blk(e, vec![i_nd, i_a, i_b, i_r])]);
        let first = find_counterexample(&b.m, Duration::from_secs(10), true).expect("first");
        assert_eq!(first.error_inst, ia);
        let opts = CexOptions {
            skip: [ia].into_iter().collect(),
            ..CexOptions::default()
        };
        let second = find_counterexample_excluding(&b.m, Duration::from_secs(10), true, &opts)
            .expect("second");
        assert_eq!(second.error_inst, ib);
        assert_eq!(second.nondets.len(), 1);
    }

    /// `y = x << 30; z = y - 1` (`nsw` sub): the prover sees the shift as a
    /// possible overflow, the counterexample search does not (the replay could
    /// never confirm it) and reports the subtraction (`y == INT_MIN`).
    #[test]
    fn counterexample_skips_shift_overflows() {
        let mut b = B::new();
        let nd = b.decl("__VERIFIER_nondet_int");
        let e = b.bid();
        let (i_nd, x) = b.inst(Operation::CallDirect { callee: nd }, vec![], Some(I32));
        let thirty = b.konst(30);
        let one = b.konst(1);
        let (i_sh, y) = b.inst(
            Operation::BinaryOp {
                kind: BinaryOp::Shl,
            },
            vec![x.unwrap(), thirty],
            Some(I32),
        );
        let (mut i_add, _) = b.inst(
            Operation::BinaryOp {
                kind: BinaryOp::Sub,
            },
            vec![y.unwrap(), one],
            Some(I32),
        );
        i_add = i_add.with_no_signed_wrap();
        let (i_r, _) = b.inst(Operation::Ret, vec![], None);
        let (ish, iadd) = (i_sh.id, i_add.id);
        b.func("main", vec![], vec![blk(e, vec![i_nd, i_sh, i_add, i_r])]);
        assert_eq!(
            prove_ovf(&b.m),
            ExhaustiveProof::Abstain("error-feasible".into())
        );
        let cex = find_counterexample(&b.m, Duration::from_secs(10), true).expect("cex");
        assert_ne!(cex.error_inst, ish);
        assert_eq!(cex.error_inst, iadd);
        // Last resort: shifts are reported too, and the shift comes first.
        let opts = CexOptions {
            check_shl: true,
            ..CexOptions::default()
        };
        let with_shl = find_counterexample_excluding(&b.m, Duration::from_secs(10), true, &opts)
            .expect("shl cex");
        assert_eq!(with_shl.error_inst, ish);
    }

    /// `y = x << 30` with a source location: told that no signed shift lives at
    /// that location, the prover proves the program (an unsigned shift wraps);
    /// told the shift IS signed, or told nothing, it reports the overflow.
    #[test]
    fn unsigned_shift_locations_are_not_overflows() {
        let mut b = B::new();
        let nd = b.decl("__VERIFIER_nondet_int");
        let e = b.bid();
        let (i_nd, x) = b.inst(Operation::CallDirect { callee: nd }, vec![], Some(I32));
        let thirty = b.konst(30);
        let (mut i_sh, _) = b.inst(
            Operation::BinaryOp {
                kind: BinaryOp::Shl,
            },
            vec![x.unwrap(), thirty],
            Some(I32),
        );
        i_sh.span = Some(point_span(10, 5));
        let (i_r, _) = b.inst(Operation::Ret, vec![], None);
        b.func("main", vec![], vec![blk(e, vec![i_nd, i_sh, i_r])]);
        b.m.source_files.push(saf_core::span::SourceFile::new(
            saf_core::ids::FileId::new(1),
            "t.c",
        ));
        let budget = Duration::from_secs(10);
        assert_eq!(
            prove_ovf(&b.m),
            ExhaustiveProof::Abstain("error-feasible".into())
        );
        let elsewhere: BTreeSet<(u32, u32)> = [(12, 7)].into_iter().collect();
        assert_eq!(
            prove_no_overflow_exhaustive_shl(&b.m, budget, Some(&elsewhere)),
            ExhaustiveProof::Proven
        );
        let here: BTreeSet<(u32, u32)> = [(10, 5)].into_iter().collect();
        assert_eq!(
            prove_no_overflow_exhaustive_shl(&b.m, budget, Some(&here)),
            ExhaustiveProof::Abstain("error-feasible".into())
        );
    }

    /// `c = getchar(); c + 1` (`nsw`): the counterexample search pins the
    /// extern's result like a nondet read (the replay driver wraps it).
    #[test]
    fn counterexample_pins_a_pinned_extern_result() {
        let mut b = B::new();
        let gc = b.decl("getchar");
        let e = b.bid();
        let (i_gc, c) = b.inst(Operation::CallDirect { callee: gc }, vec![], Some(I32));
        let one = b.konst(1);
        let (mut i_add, _) = b.inst(
            Operation::BinaryOp {
                kind: BinaryOp::Add,
            },
            vec![c.unwrap(), one],
            Some(I32),
        );
        i_add = i_add.with_no_signed_wrap();
        let (i_r, _) = b.inst(Operation::Ret, vec![], None);
        let gc_id = i_gc.id;
        b.func("main", vec![], vec![blk(e, vec![i_gc, i_add, i_r])]);
        let cex = find_counterexample(&b.m, Duration::from_secs(10), true).expect("cex");
        assert_eq!(cex.nondets.len(), 1);
        assert_eq!(cex.nondets[0].0, "getchar");
        assert_eq!(cex.nondets[0].1, i64::from(i32::MAX));
        assert_eq!(cex.nondets[0].2, gc_id);
    }

    /// `foo((void *)nondet()); reach_error();` — an unknown extern with a
    /// pointer argument the engine cannot name: the prover abstains (the callee
    /// may write anywhere), the replay-gated counterexample search goes on.
    #[test]
    fn counterexample_search_tolerates_opaque_extern_pointer_args() {
        let mut b = B::new();
        let err = b.decl("reach_error");
        let foo = b.decl("foo");
        let nd = b.decl("__VERIFIER_nondet_int");
        let e = b.bid();
        let (i_nd, n) = b.inst(Operation::CallDirect { callee: nd }, vec![], Some(I32));
        let (i_cast, p) = b.inst(
            Operation::Cast {
                kind: CastKind::IntToPtr,
                target_bits: None,
            },
            vec![n.unwrap()],
            Some(PTR),
        );
        let (i_foo, _) = b.inst(
            Operation::CallDirect { callee: foo },
            vec![p.unwrap()],
            None,
        );
        let (i_err, _) = b.inst(Operation::CallDirect { callee: err }, vec![], None);
        let (i_r, _) = b.inst(Operation::Ret, vec![], None);
        let err_id = i_err.id;
        b.func(
            "main",
            vec![],
            vec![blk(e, vec![i_nd, i_cast, i_foo, i_err, i_r])],
        );
        assert_eq!(
            prove(&b.m),
            ExhaustiveProof::Abstain("extern-foo-opaque-pointer-arg".into())
        );
        let cex = find_counterexample(&b.m, Duration::from_secs(10), false).expect("cex");
        assert_eq!(cex.error_inst, err_id);
    }

    /// `x = ldv_undef_int(); if (x == 7) reach_error();` — the result of a
    /// function the task never defines is pinned like a nondet read (the
    /// replay driver stubs it with the model's value).
    #[test]
    fn counterexample_pins_undefined_extern_results() {
        let mut b = B::new();
        let err = b.decl("reach_error");
        let ud = b.decl("ldv_undef_int");
        let (e, bb_then, bb_end) = (b.bid(), b.bid(), b.bid());
        let (i_ud, x) = b.inst(Operation::CallDirect { callee: ud }, vec![], Some(I32));
        let ud_id = i_ud.id;
        let seven = b.konst(7);
        let (i_c, c) = b.inst(
            Operation::BinaryOp {
                kind: BinaryOp::ICmpEq,
            },
            vec![x.unwrap(), seven],
            Some(I1),
        );
        let (i_br, _) = b.inst(
            Operation::CondBr {
                then_target: bb_then,
                else_target: bb_end,
            },
            vec![c.unwrap()],
            None,
        );
        let (i_err, _) = b.inst(Operation::CallDirect { callee: err }, vec![], None);
        let (i_r1, _) = b.inst(Operation::Ret, vec![], None);
        let (i_r2, _) = b.inst(Operation::Ret, vec![], None);
        b.func(
            "main",
            vec![],
            vec![
                blk(e, vec![i_ud, i_c, i_br]),
                blk(bb_then, vec![i_err, i_r1]),
                blk(bb_end, vec![i_r2]),
            ],
        );
        let cex = find_counterexample(&b.m, Duration::from_secs(10), false).expect("cex");
        assert_eq!(cex.nondets, vec![("ldv_undef_int".to_string(), 7, ud_id)]);
    }

    /// Float programs: `x = nondet_double()`, then `body(x)` decides what is
    /// compared and whether `reach_error` is reached.
    fn float_program(
        body: impl FnOnce(&mut B, ValueId) -> (Vec<Instruction>, ValueId),
    ) -> AirModule {
        let mut b = B::new();
        let err = b.decl("reach_error");
        let nd = b.decl("__VERIFIER_nondet_double");
        let (e, bb_then, bb_end) = (b.bid(), b.bid(), b.bid());
        let (i_nd, x) = b.inst(Operation::CallDirect { callee: nd }, vec![], Some(F64));
        let (mut insts, cond) = body(&mut b, x.unwrap());
        let (i_br, _) = b.inst(
            Operation::CondBr {
                then_target: bb_then,
                else_target: bb_end,
            },
            vec![cond],
            None,
        );
        let (i_err, _) = b.inst(Operation::CallDirect { callee: err }, vec![], None);
        let (i_r1, _) = b.inst(Operation::Ret, vec![], None);
        let (i_r2, _) = b.inst(Operation::Ret, vec![], None);
        let mut entry = vec![i_nd];
        entry.append(&mut insts);
        entry.push(i_br);
        b.func(
            "main",
            vec![],
            vec![
                blk(e, entry),
                blk(bb_then, vec![i_err, i_r1]),
                blk(bb_end, vec![i_r2]),
            ],
        );
        b.m
    }

    fn fbin(
        b: &mut B,
        kind: BinaryOp,
        x: ValueId,
        y: ValueId,
        ty: TypeId,
    ) -> (Instruction, ValueId) {
        let (i, v) = b.inst(Operation::BinaryOp { kind }, vec![x, y], Some(ty));
        (i, v.unwrap())
    }

    /// `x > 1.5 && x < 1.0` is unsatisfiable over IEEE doubles: proven.
    #[test]
    fn float_contradictory_bounds_are_proven() {
        let m = float_program(|b, x| {
            let (c15, c10) = (b.fkonst(1.5), b.fkonst(1.0));
            let (i1, gt) = fbin(b, BinaryOp::FCmpOgt, x, c15, I1);
            let (i2, lt) = fbin(b, BinaryOp::FCmpOlt, x, c10, I1);
            let (i3, both) = fbin(b, BinaryOp::And, gt, lt, I1);
            (vec![i1, i2, i3], both)
        });
        assert_eq!(prove(&m), ExhaustiveProof::Proven);
    }

    /// `x != x` (unordered) is true for NaN — the error is feasible; the
    /// ordered `one` of a value with itself never is.
    #[test]
    fn float_nan_is_modelled() {
        let une = float_program(|b, x| {
            let (i, c) = fbin(b, BinaryOp::FCmpUne, x, x, I1);
            (vec![i], c)
        });
        assert_eq!(
            prove(&une),
            ExhaustiveProof::Abstain("error-feasible".into())
        );
        let one = float_program(|b, x| {
            let (i, c) = fbin(b, BinaryOp::FCmpOne, x, x, I1);
            (vec![i], c)
        });
        assert_eq!(prove(&one), ExhaustiveProof::Proven);
    }

    /// `x + 1.0 == x` holds for |x| ≥ 2^53 (and infinities): IEEE rounding is
    /// exact, so the error is feasible — a real-number model would prove it.
    #[test]
    fn float_addition_rounds_like_ieee() {
        let m = float_program(|b, x| {
            let one = b.fkonst(1.0);
            let (i1, y) = fbin(b, BinaryOp::FAdd, x, one, F64);
            let (i2, c) = fbin(b, BinaryOp::FCmpOeq, y, x, I1);
            (vec![i1, i2], c)
        });
        assert_eq!(prove(&m), ExhaustiveProof::Abstain("error-feasible".into()));
    }

    /// `0.0 <= x <= 10.5` then `(int)x > 10` is impossible (truncation toward
    /// zero), and the conversion is defined in that range: proven. Without the
    /// bounds the conversion of a huge value is any integer: feasible.
    #[test]
    fn float_to_int_truncates_within_range() {
        let bounded = float_program(|b, x| {
            let (lo, hi, ten) = (b.fkonst(0.0), b.fkonst(10.5), b.konst(10));
            let (i1, ge) = fbin(b, BinaryOp::FCmpOge, x, lo, I1);
            let (i2, le) = fbin(b, BinaryOp::FCmpOle, x, hi, I1);
            let (i3, i) = b.inst(
                Operation::Cast {
                    kind: CastKind::FPToSI,
                    target_bits: Some(32),
                },
                vec![x],
                Some(I32),
            );
            let (i4, big) = fbin(b, BinaryOp::ICmpSgt, i.unwrap(), ten, I1);
            let (i5, a) = fbin(b, BinaryOp::And, ge, le, I1);
            let (i6, all) = fbin(b, BinaryOp::And, a, big, I1);
            (vec![i1, i2, i3, i4, i5, i6], all)
        });
        assert_eq!(prove(&bounded), ExhaustiveProof::Proven);
        let unbounded = float_program(|b, x| {
            let ten = b.konst(10);
            let (i3, i) = b.inst(
                Operation::Cast {
                    kind: CastKind::FPToSI,
                    target_bits: Some(32),
                },
                vec![x],
                Some(I32),
            );
            let (i4, big) = fbin(b, BinaryOp::ICmpSgt, i.unwrap(), ten, I1);
            (vec![i3, i4], big)
        });
        assert_eq!(
            prove(&unbounded),
            ExhaustiveProof::Abstain("error-feasible".into())
        );
    }

    /// `float f = nondet_float(); if ((double)f == 1.0) reach_error();` — the
    /// widening conversion is modelled and the error is feasible (`f == 1.0f`);
    /// narrowing back to `float` and comparing with the original never differs.
    #[test]
    fn float_widening_and_narrowing_are_modelled() {
        let mut b = B::new();
        let err = b.decl("reach_error");
        let nd = b.decl("__VERIFIER_nondet_float");
        let (e, bb_then, bb_end) = (b.bid(), b.bid(), b.bid());
        let (i_nd, f) = b.inst(Operation::CallDirect { callee: nd }, vec![], Some(F32));
        let (i_ext, d) = b.inst(
            Operation::Cast {
                kind: CastKind::FPExt,
                target_bits: None,
            },
            vec![f.unwrap()],
            Some(F64),
        );
        let one = b.fkonst(1.0);
        let (i_c, c) = fbin(&mut b, BinaryOp::FCmpOeq, d.unwrap(), one, I1);
        let (i_br, _) = b.inst(
            Operation::CondBr {
                then_target: bb_then,
                else_target: bb_end,
            },
            vec![c],
            None,
        );
        let (i_err, _) = b.inst(Operation::CallDirect { callee: err }, vec![], None);
        let (i_r1, _) = b.inst(Operation::Ret, vec![], None);
        let (i_r2, _) = b.inst(Operation::Ret, vec![], None);
        b.func(
            "main",
            vec![],
            vec![
                blk(e, vec![i_nd, i_ext, i_c, i_br]),
                blk(bb_then, vec![i_err, i_r1]),
                blk(bb_end, vec![i_r2]),
            ],
        );
        assert_eq!(
            prove(&b.m),
            ExhaustiveProof::Abstain("error-feasible".into())
        );
    }

    /// The raw FPA helpers on constants: `(double)1.0f`, `(double)1`, `(int)2.5`.
    #[test]
    fn fpa_helpers_convert_constants() {
        let one_f = int_bv(i64::from(1.0f32.to_bits()), 32);
        let widened = fpa::convert(&one_f, 64).expect("fpext");
        assert_eq!(
            widened.simplify().as_u64(),
            Some(1.0f64.to_bits()),
            "fpext 1.0f"
        );
        let from_int = fpa::from_int(&int_bv(1, 32), true, 64).expect("sitofp");
        assert_eq!(
            from_int.simplify().as_u64(),
            Some(1.0f64.to_bits()),
            "sitofp 1"
        );
        let two5 = int_bv(i64::from_ne_bytes(2.5f64.to_bits().to_ne_bytes()), 64);
        let (conv, in_range) = fpa::to_int(&two5, true, 32).expect("fptosi");
        assert_eq!(conv.simplify().as_u64(), Some(2), "fptosi 2.5");
        assert_eq!(in_range.simplify().as_bool(), Some(true));
    }

    /// `if (x > INT_MIN && abs(x) < 46340) { y = x * x; }` — `abs` is
    /// `llvm.abs.i32`, evaluated only once `x > INT_MIN` holds (C's `&&`);
    /// the square then cannot overflow: proven. Without that guard
    /// `abs(INT_MIN)` is itself the overflow.
    #[test]
    fn integer_abs_builtin_is_exact_and_checks_int_min() {
        fn program(guard_min: bool) -> AirModule {
            let mut b = B::new();
            let nd = b.decl("__VERIFIER_nondet_int");
            let abs = b.decl("llvm.abs.i32");
            let (e, bb_abs, bb_sq, bb_end) = (b.bid(), b.bid(), b.bid(), b.bid());
            let (i_nd, x) = b.inst(Operation::CallDirect { callee: nd }, vec![], Some(I32));
            let x = x.unwrap();
            let min = b.konst(i64::from(i32::MIN));
            let (i_gt, above) = fbin(&mut b, BinaryOp::ICmpSgt, x, min, I1);
            let (i_br0, _) = b.inst(
                Operation::CondBr {
                    then_target: bb_abs,
                    else_target: bb_end,
                },
                vec![above],
                None,
            );
            let poison = b.konst(1);
            let (i_abs, ax) = b.inst(
                Operation::CallDirect { callee: abs },
                vec![x, poison],
                Some(I32),
            );
            let lim = b.konst(46340);
            let (i_lt, small) = fbin(&mut b, BinaryOp::ICmpSlt, ax.unwrap(), lim, I1);
            let (i_br, _) = b.inst(
                Operation::CondBr {
                    then_target: bb_sq,
                    else_target: bb_end,
                },
                vec![small],
                None,
            );
            let (mut i_mul, _) = b.inst(
                Operation::BinaryOp {
                    kind: BinaryOp::Mul,
                },
                vec![x, x],
                Some(I32),
            );
            i_mul = i_mul.with_no_signed_wrap();
            let (i_r1, _) = b.inst(Operation::Ret, vec![], None);
            let (i_r2, _) = b.inst(Operation::Ret, vec![], None);
            let (i_jmp, _) = b.inst(Operation::Br { target: bb_abs }, vec![], None);
            let entry = if guard_min {
                vec![i_nd, i_gt, i_br0]
            } else {
                vec![i_nd, i_jmp]
            };
            b.func(
                "main",
                vec![],
                vec![
                    blk(e, entry),
                    blk(bb_abs, vec![i_abs, i_lt, i_br]),
                    blk(bb_sq, vec![i_mul, i_r1]),
                    blk(bb_end, vec![i_r2]),
                ],
            );
            b.m
        }
        assert_eq!(prove_ovf(&program(true)), ExhaustiveProof::Proven);
        assert_eq!(
            prove_ovf(&program(false)),
            ExhaustiveProof::Abstain("error-feasible".into())
        );
    }

    /// `(long)sqrt(2147483647.0) == 46340` — `sqrt` of a constant is a
    /// constant; `if ((int)sqrt(c) != 46340) reach_error();` is proven.
    #[test]
    fn sqrt_builtin_folds_constants() {
        let mut b = B::new();
        let err = b.decl("reach_error");
        let sqrt = b.decl("sqrt");
        let (e, bb_then, bb_end) = (b.bid(), b.bid(), b.bid());
        let c = b.fkonst(2_147_483_647.0);
        let (i_sq, r) = b.inst(Operation::CallDirect { callee: sqrt }, vec![c], Some(F64));
        let (i_cast, i) = b.inst(
            Operation::Cast {
                kind: CastKind::FPToSI,
                target_bits: Some(32),
            },
            vec![r.unwrap()],
            Some(I32),
        );
        let k = b.konst(46340);
        let (i_ne, ne) = fbin(&mut b, BinaryOp::ICmpNe, i.unwrap(), k, I1);
        let (i_br, _) = b.inst(
            Operation::CondBr {
                then_target: bb_then,
                else_target: bb_end,
            },
            vec![ne],
            None,
        );
        let (i_err, _) = b.inst(Operation::CallDirect { callee: err }, vec![], None);
        let (i_r1, _) = b.inst(Operation::Ret, vec![], None);
        let (i_r2, _) = b.inst(Operation::Ret, vec![], None);
        b.func(
            "main",
            vec![],
            vec![
                blk(e, vec![i_sq, i_cast, i_ne, i_br]),
                blk(bb_then, vec![i_err, i_r1]),
                blk(bb_end, vec![i_r2]),
            ],
        );
        assert_eq!(prove(&b.m), ExhaustiveProof::Proven);
    }

    /// x87 `long double` round trip: `(double)(long double)x == x` for every
    /// double that compares equal to itself (not NaN) — proven, the 64-bit
    /// significand holds a double exactly; and `(long double)1` has the x87
    /// encoding `0x3fff` / `0x8000000000000000` (explicit integer bit set).
    #[test]
    fn long_double_is_repacked_exactly() {
        const F80: TypeId = TypeId::new(800);
        let mut b = B::new();
        b.m.types.insert(F80, AirType::Float { bits: 80 });
        let err = b.decl("reach_error");
        let nd = b.decl("__VERIFIER_nondet_double");
        let (e, bb_then, bb_end) = (b.bid(), b.bid(), b.bid());
        let (i_nd, x) = b.inst(Operation::CallDirect { callee: nd }, vec![], Some(F64));
        let x = x.unwrap();
        let (i_ext, w) = b.inst(
            Operation::Cast {
                kind: CastKind::FPExt,
                target_bits: None,
            },
            vec![x],
            Some(F80),
        );
        let (i_tr, back) = b.inst(
            Operation::Cast {
                kind: CastKind::FPTrunc,
                target_bits: None,
            },
            vec![w.unwrap()],
            Some(F64),
        );
        let (i_ord, ord) = fbin(&mut b, BinaryOp::FCmpOeq, x, x, I1);
        let (i_ne, ne) = fbin(&mut b, BinaryOp::FCmpOne, back.unwrap(), x, I1);
        let (i_and, bad) = fbin(&mut b, BinaryOp::And, ord, ne, I1);
        let (i_br, _) = b.inst(
            Operation::CondBr {
                then_target: bb_then,
                else_target: bb_end,
            },
            vec![bad],
            None,
        );
        let (i_err, _) = b.inst(Operation::CallDirect { callee: err }, vec![], None);
        let (i_r1, _) = b.inst(Operation::Ret, vec![], None);
        let (i_r2, _) = b.inst(Operation::Ret, vec![], None);
        b.func(
            "main",
            vec![],
            vec![
                blk(e, vec![i_nd, i_ext, i_tr, i_ord, i_ne, i_and, i_br]),
                blk(bb_then, vec![i_err, i_r1]),
                blk(bb_end, vec![i_r2]),
            ],
        );
        assert_eq!(prove(&b.m), ExhaustiveProof::Proven);
        let one = fpa::from_int(&int_bv(1, 32), true, 80).expect("sitofp to long double");
        let hi = one.extract(79, 64).simplify().as_u64();
        let lo = one.extract(63, 0).simplify().as_u64();
        assert_eq!((hi, lo), (Some(0x3fff), Some(0x8000_0000_0000_0000)));
    }

    /// The square bound is floor(sqrt(MAX)) for every width (46340² fits an
    /// `int`, 46341² does not).
    #[test]
    fn square_overflow_bound_is_the_integer_square_root() {
        assert_eq!(isqrt(i128::from(i32::MAX)), 46340);
        assert_eq!(isqrt(i128::from(i64::MAX)), 3_037_000_499);
        assert_eq!(isqrt(127), 11);
        assert_eq!(isqrt(0), 0);
    }

    /// `x * x` with `x` a 64-bit nondet: feasible unguarded; proven once
    /// `|x| <= 3037000499` — the square encoding makes this a cheap check.
    #[test]
    fn square_of_bounded_int64_is_proven() {
        fn program(guarded: bool) -> AirModule {
            let mut b = B::new();
            const I64: TypeId = TypeId::new(64);
            b.m.types.insert(I64, AirType::Integer { bits: 64 });
            let nd = b.decl("__VERIFIER_nondet_long");
            let (e, bb_sq, bb_end) = (b.bid(), b.bid(), b.bid());
            let (i_nd, x) = b.inst(Operation::CallDirect { callee: nd }, vec![], Some(I64));
            let x = x.unwrap();
            let lim = ValueId::new(b.id());
            b.m.constants.insert(
                lim,
                Constant::Int {
                    value: 3_037_000_499,
                    bits: 64,
                },
            );
            let nlim = ValueId::new(b.id());
            b.m.constants.insert(
                nlim,
                Constant::Int {
                    value: -3_037_000_499,
                    bits: 64,
                },
            );
            let (i_le, le) = fbin(&mut b, BinaryOp::ICmpSle, x, lim, I1);
            let (i_ge, ge) = fbin(&mut b, BinaryOp::ICmpSge, x, nlim, I1);
            let (i_and, ok) = fbin(&mut b, BinaryOp::And, le, ge, I1);
            let one = b.konst(1);
            let guard = if guarded { ok } else { one };
            let (i_br, _) = b.inst(
                Operation::CondBr {
                    then_target: bb_sq,
                    else_target: bb_end,
                },
                vec![guard],
                None,
            );
            let (mut i_mul, _) = b.inst(
                Operation::BinaryOp {
                    kind: BinaryOp::Mul,
                },
                vec![x, x],
                Some(I64),
            );
            i_mul = i_mul.with_no_signed_wrap();
            let (i_r1, _) = b.inst(Operation::Ret, vec![], None);
            let (i_r2, _) = b.inst(Operation::Ret, vec![], None);
            b.func(
                "main",
                vec![],
                vec![
                    blk(e, vec![i_nd, i_le, i_ge, i_and, i_br]),
                    blk(bb_sq, vec![i_mul, i_r1]),
                    blk(bb_end, vec![i_r2]),
                ],
            );
            b.m
        }
        assert_eq!(prove_ovf(&program(true)), ExhaustiveProof::Proven);
        assert_eq!(
            prove_ovf(&program(false)),
            ExhaustiveProof::Abstain("error-feasible".into())
        );
    }

    /// A `calloc`ed (or `kzalloc`ed) cell reads 0 until written: `if (*p != 0)
    /// reach_error();` is proven; the same program with `malloc` is feasible.
    #[test]
    fn zero_allocated_heap_cells_read_zero() {
        use saf_core::air::HeapAllocKind;
        fn program(kind: HeapAllocKind) -> AirModule {
            let mut b = B::new();
            let err = b.decl("reach_error");
            let (e, bb_then, bb_end) = (b.bid(), b.bid(), b.bid());
            let size = b.konst(4);
            let (i_p, p) = b.inst(Operation::HeapAlloc { kind }, vec![size], Some(PTR));
            let (i_ld, v) = b.inst(Operation::Load, vec![p.unwrap()], Some(I32));
            let zero = b.konst(0);
            let (i_ne, ne) = fbin(&mut b, BinaryOp::ICmpNe, v.unwrap(), zero, I1);
            let (i_br, _) = b.inst(
                Operation::CondBr {
                    then_target: bb_then,
                    else_target: bb_end,
                },
                vec![ne],
                None,
            );
            let (i_err, _) = b.inst(Operation::CallDirect { callee: err }, vec![], None);
            let (i_r1, _) = b.inst(Operation::Ret, vec![], None);
            let (i_r2, _) = b.inst(Operation::Ret, vec![], None);
            b.func(
                "main",
                vec![],
                vec![
                    blk(e, vec![i_p, i_ld, i_ne, i_br]),
                    blk(bb_then, vec![i_err, i_r1]),
                    blk(bb_end, vec![i_r2]),
                ],
            );
            b.m
        }
        assert_eq!(
            prove(&program(HeapAllocKind::Calloc)),
            ExhaustiveProof::Proven
        );
        assert_eq!(
            prove(&program(HeapAllocKind::Other("kzalloc".into()))),
            ExhaustiveProof::Proven
        );
        assert_eq!(
            prove(&program(HeapAllocKind::Malloc)),
            ExhaustiveProof::Abstain("error-feasible".into())
        );
    }

    /// `struct s *p = kzalloc(..); if (p->next != NULL) reach_error();` — the
    /// pointer field of a zero-allocated object reads NULL: proven; with
    /// `kmalloc` the field is anything: feasible.
    #[test]
    fn pointer_field_of_zero_allocated_object_is_null() {
        use saf_core::air::HeapAllocKind;
        fn program(kind: HeapAllocKind) -> AirModule {
            let mut b = B::new();
            let err = b.decl("reach_error");
            let (e, bb_then, bb_end) = (b.bid(), b.bid(), b.bid());
            let size = b.konst(8);
            let (i_p, p) = b.inst(Operation::HeapAlloc { kind }, vec![size], Some(PTR));
            let (i_ld, q) = b.inst(Operation::Load, vec![p.unwrap()], Some(PTR));
            let null = ValueId::new(b.id());
            b.m.constants.insert(null, Constant::Null);
            let (i_ne, ne) = fbin(&mut b, BinaryOp::ICmpNe, q.unwrap(), null, I1);
            let (i_br, _) = b.inst(
                Operation::CondBr {
                    then_target: bb_then,
                    else_target: bb_end,
                },
                vec![ne],
                None,
            );
            let (i_err, _) = b.inst(Operation::CallDirect { callee: err }, vec![], None);
            let (i_r1, _) = b.inst(Operation::Ret, vec![], None);
            let (i_r2, _) = b.inst(Operation::Ret, vec![], None);
            b.func(
                "main",
                vec![],
                vec![
                    blk(e, vec![i_p, i_ld, i_ne, i_br]),
                    blk(bb_then, vec![i_err, i_r1]),
                    blk(bb_end, vec![i_r2]),
                ],
            );
            b.m
        }
        assert_eq!(
            prove(&program(HeapAllocKind::Other("kzalloc".into()))),
            ExhaustiveProof::Proven
        );
        assert_eq!(
            prove(&program(HeapAllocKind::Other("kmalloc".into()))),
            ExhaustiveProof::Abstain("error-feasible".into())
        );
    }

    /// A task that DEFINES `kfree` (here: a body that reaches the error) is
    /// executed, not modelled as the no-op extern of the same name.
    #[test]
    fn defined_function_wins_over_the_extern_model_of_its_name() {
        let mut b = B::new();
        let err = b.decl("reach_error");
        let (kb, e) = (b.bid(), b.bid());
        let (i_err, _) = b.inst(Operation::CallDirect { callee: err }, vec![], None);
        let (i_kr, _) = b.inst(Operation::Ret, vec![], None);
        let kfree = b.func("kfree", vec![], vec![blk(kb, vec![i_err, i_kr])]);
        let (i_call, _) = b.inst(Operation::CallDirect { callee: kfree }, vec![], None);
        let (i_r, _) = b.inst(Operation::Ret, vec![], None);
        b.func("main", vec![], vec![blk(e, vec![i_call, i_r])]);
        assert_eq!(
            prove(&b.m),
            ExhaustiveProof::Abstain("error-feasible".into())
        );
    }
}
