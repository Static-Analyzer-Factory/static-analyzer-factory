# Plan 189 — Research Notes: Peer Tool Capabilities

This file backs every comparison-table claim with a citation. Last verified
2026-04-27.

## Methodology

For each peer tool (SVF, Lotus, Phasar, CodeQL, Infer), the 13 dimensions
listed in `plans/189-site-repositioning-and-comparison.md` were verified
against:

- The project's **GitHub README** (primary).
- The project's **license file** (for explicit license names).
- The project's **official documentation site** when the README was thin.
- The project's **`include/` and `lib/` source-tree listings** to verify
  shipped solver classes and checker counts (added 2026-04-27 after a
  user request for code-level verification).
- **WebSearch** as a fallback for less-documented capabilities.

For SAF itself, claims were verified by directly inspecting
`crates/saf-analysis/src/{pta,fspta,cspta,dda,svfg,valueflow,ifds,checkers,mssa,absint}/`
and the shipped specs at `share/saf/specs/{taint,libc,posix,gnu,cxxabi}/`.

When a dimension could not be confirmed from primary sources, the entry is
marked **"unconfirmed"** rather than asserted negatively. Where a tool
*partially* supports a feature (e.g., via a converter or a separate package),
the entry says so — never a flat "unsupported."

## SVF

- Repo: https://github.com/SVF-tools/SVF
- License: **GNU Affero General Public License v3.0** —
  https://github.com/SVF-tools/SVF/blob/master/LICENSE.TXT
- Target IR: **LLVM IR** ("static analysis tool for LLVM-based languages",
  C/C++ in practice).
- Pointer analysis: Andersen CI, **flow-sensitive (FS)** [CGO'21, OOPSLA'21],
  **field-sensitive** [SAS'19], **context-sensitive (CS)**, **demand-driven
  (DDA)** — full set. Source: SVF README "What is SVF?" section.
  - Code verification (2026-04-27): `svf/include/WPA/` ships `Andersen.h`,
    `AndersenPWC.h`, `CSC.h`, `FlowSensitive.h`, `Steensgaard.h`,
    `TypeAnalysis.h`, `VersionedFlowSensitive.h`. `svf/include/DDA/`
    ships `ContextDDA.h`, `DDAClient.h`, `DDAPass.h`, `DDAStat.h`,
    `DDAVFSolver.h`, `FlowDDA.h` — DDA is first-class with both flow- and
    context-sensitive variants. Top-level `svf/include/` also has `AE`
    (abstract execution), `MSSA`, `MTA`, `CFL`, `MemoryModel` — SVF is
    broader than just PTA.
- Value-flow / SVFG: **Yes — the project's headline capability.** README:
  "generating a variety of graphs, including call graph, ICFG, class
  hierarchy graph, constraint graph, value-flow graph."
- IFDS / IDE solver: **Not built-in.** SVF's data-flow reasoning is
  value-flow-graph-based, not IFDS.
- Taint analysis: **Not built-in as a first-class checker.** The SABER
  component focuses on memory leaks, double-free, and file checking. Taint
  can be implemented on top of SVFG by users.
- Built-in checkers: **3 SABER checkers** verified by listing
  `svf/include/SABER/`: `LeakChecker.h` (memory leaks),
  `DoubleFreeChecker.h` (double-free), `FileChecker.h` (file-handle bugs).
  Plus supporting infrastructure (ProgSlice, SaberCheckerAPI,
  SaberSVFGBuilder, SrcSnkSolver). No additional checker library outside
  SABER.
- SARIF export: **Not standard.** No SARIF support found in README or
  documentation.
- Primary SDK: **C++** (96.5% of the codebase). A separate Python binding
  package `Pysvf` exists at https://github.com/SVF-tools/SVF-Python — it
  wraps the C++ core (PAG/SVFIR, ICFG, SVFG, Andersen, Steensgaard, MTA).
  → Comparison entry: "C++ primary, Python via separate Pysvf wrapper."
- LLVM versions: **Single per build.** Current HEAD targets LLVM 16
  (README: "SVF now supports LLVM-16.0.0 with opaque pointers"). Older
  versions (LLVM 4–13) live on historical branches. SVF does not ship
  multiple LLVM tags simultaneously.
- Browser / WASM: **No.**
- Determinism: **Not advertised.**
- AI-agent / coding-agent skills: **No** (no `.claude` or `AGENTS.md` skill
  pack found).

## Phasar

- Repo: https://github.com/secure-software-engineering/phasar
- Site: https://phasar.org/
- License: **MIT.**
- Target IR: **LLVM IR.** "A LLVM-based static analysis framework written in
  C++."
- Pointer analysis: provides points-to information as a helper analysis
  ("Computing points-to information, call-graph(s), etc. is done by the
  framework"). Variants not enumerated in README — listed as "computed
  internally" rather than as a user-facing taxonomy.
  → Comparison entry: "Helper analyses (points-to, CG)" without claiming
    the full FS/CS/DDA matrix.
- Value-flow / SVFG: **Not in the SVFG paradigm.** Phasar's data-flow
  reasoning is IFDS/IDE-based.
- IFDS / IDE solver: **Yes — Phasar's specialty.** "Allows users to specify
  arbitrary data-flow problems which are then solved in a fully-automated
  manner on the specified LLVM IR target code." Phasar's contribution to the
  literature is the IFDS/IDE solver.
  - Code verification (2026-04-27): `include/phasar/DataFlow/` has
    `IfdsIde/`, `Mono/`, `PathSensitivity/` subdirectories. The `IfdsIde/`
    subdirectory has the full solver scaffolding (`IFDSTabulationProblem.h`,
    `IDETabulationProblem.h`, `EdgeFunction.h`, `FlowFunctions.h`,
    `IFDSIDESolverConfig.h`, `Solver/` subdirectory, etc.).
- Taint analysis: **Yes** (built-in IFDS and IDE taint clients).
- Built-in checkers: **15+ IFDS/IDE clients** verified by listing
  `include/phasar/PhasarLLVM/DataFlow/IfdsIde/Problems/`:
  - **IFDS clients (7):** `IFDSConstAnalysis.h`, `IFDSProtoAnalysis.h`,
    `IFDSSignAnalysis.h`, `IFDSSolverTest.h`, `IFDSTaintAnalysis.h`,
    `IFDSTypeAnalysis.h`, `IFDSUninitializedVariables.h`.
  - **IDE clients (8):** `IDEExtendedTaintAnalysis.h`,
    `IDEFeatureTaintAnalysis.h`, `IDEInstInteractionAnalysis.h`,
    `IDELinearConstantAnalysis.h`, `IDEProtoAnalysis.h`,
    `IDESecureHeapPropagation.h`, `IDESolverTest.h`,
    `IDETypeStateAnalysis.h`.
  - Plus subdirectories `ExtendedTaintAnalysis/`, `IDEGeneralizedLCA/`,
    `TypeStateDescriptions/`. Phasar is *framework-first* but also ships a
    rich catalog of canned analyses — my earlier "few" claim was wrong and
    has been corrected.
- SARIF export: **Not natively.**
- Primary SDK: **C++ only** (C++20 minimum). No Python binding.
- LLVM versions: **LLVM 16 and 17** (two), per README: "PhASAR is currently
  set up to support LLVM-16 and 17, using LLVM 16 by default."
- Browser / WASM: **No.**
- Determinism: **Not advertised.**
- AI-agent / coding-agent skills: **No.**

## Lotus

- Repo: https://github.com/ZJU-PL/lotus
- Docs: https://zju-pl.github.io/lotus/user_guide/major_components.html
- License: **MIT** (core; project is multi-licensed because it bundles LLVM,
  Boost, Sea-DSA, SparrowAA, CLAM, Crab, Phasar). Source:
  https://github.com/ZJU-PL/lotus/blob/main/LICENSE
- Target IR: **LLVM IR** (LLVM 14.x).
- Pointer / alias analyses: very rich catalog. Verified by listing
  `lib/Alias/` (2026-04-27):
  - `DemandDriven/DDA/` — demand-driven analyses
  - `Dynamic/` — runtime-based alias validators
  - `InclusionBased/` — `AserPTA`, `SparrowAA`, `LotusAA`, `TPA`,
    `CclyzerAA`
  - `Specialized/` — `AllocAA`, `DFPA`, `FPA`, `SRAA`, `TypeQualifier`,
    `UnderApproxAA`
  - `UnificationBased/` — `DyckAA`, `seadsa`
  - `Infrastructure/` — shared support layers
  → Comparison entry: "Multiple bundled — extensive catalog of bundled
    third-party + first-party analyses."
- Value-flow / SVFG: **DyckVFG** — value-flow graph variant.
- IFDS / IDE solver: **Inherits Phasar bundling** but framework-emphasis is
  abstract interpretation. Lotus also has its own `lib/Solvers/` and
  `lib/Dataflow/` directories.
- Taint analysis: **Partial — via KINT** (integer bug detection with taint
  analysis and SMT). Not a generic taint framework like
  SAF/CodeQL/Phasar. Earlier "no" claim has been corrected to "partial".
- Built-in checkers: **Many.** Verified by listing `lib/Checker/`
  (2026-04-27):
  - `AE/` — Abstract Execution engine
  - `Concurrency/` — Atomicity, ConditionVariable, DataRace, Deadlock,
    LockMismatch
  - `FiTx/` — double-free, double-lock/unlock, leak, null-ptr, ref/unref,
    UAF, use-before-init
  - `KINT/` — Integer bug detection with taint analysis and SMT
  - `Pulse/` — Biabductive analysis (Lotus's own — not Infer's Pulse)
  - `Saber/` — Source-sink bug detector (memory leaks, double-free, file)
  - `Security/` — Spectre + other security detectors
  - `Report/` — shared reporting infrastructure
  Earlier "few (null exception)" was a substantial undercount and has been
  corrected.
- SARIF export: **Not mentioned.**
- Primary SDK: **C++** (82.7% of codebase). No Python binding mentioned.
- LLVM versions: **LLVM 14.x only** (single).
- Browser / WASM: **No.**
- Determinism: **Not advertised.**
- AI-agent / coding-agent skills: An `AGENTS.md` file is referenced in the
  repo, but its scope and depth aren't documented. Mark as
  **"AGENTS.md present, scope unclear"** rather than asserting full skill
  integration.

## Infer

- Repo: https://github.com/facebook/infer
- License: **MIT.**
- Target languages: **C, C++, Objective-C, Java** (also C# via Pulse for
  some checkers). **Not LLVM-IR-based** — uses Clang AST and other
  language-specific frontends.
- Pointer analysis: not Andersen-style PTA. Pulse uses
  separation-logic-style abstract interpretation. RacerD has its own memory
  model. So the "Andersen / FS / CS / DDA" axis doesn't translate directly —
  comparison entry: "Different paradigm (separation logic, abstract
  interpretation)."
- Value-flow / SVFG: **Not in the SVFG paradigm.**
- IFDS / IDE solver: **Not the primary mechanism.** Infer ships **Infer.AI**,
  a framework for building abstract-interpretation-based checkers
  (intraprocedural and interprocedural).
- Taint analysis: **Yes** — **Quandary** for C/C++/Obj-C/Java; Pulse also
  has taint sources/sinks/sanitizers. Source:
  https://github.com/facebook/infer/blob/main/website/versioned_docs/version-1.2.0/checker-quandary.md
- Built-in checkers: **Large.** Pulse (memory safety), RacerD (race
  conditions), InferBO (buffer overrun), SIOF, Quandary, Starvation, Litho,
  Lineage, Purity, ScopeLeakage, etc. ~30+ named issue types just in the
  security and memory categories — see `IBase/IssueType` reference.
- SARIF export: **Via converter** (SARIF SDK includes an Infer-format
  adapter). **Not native.** Source:
  https://github.com/facebook/infer/issues/1274
- Primary SDK: **OCaml** (Infer itself is implemented in OCaml; user
  extensions go through the OCaml-based Infer.AI framework).
- LLVM versions: **N/A** (doesn't use LLVM IR).
- Browser / WASM: **No.**
- Determinism: **Not advertised.**
- AI-agent / coding-agent skills: **No.**

## CodeQL

- Repo: https://github.com/github/codeql (queries) and
  https://github.com/github/codeql-cli-binaries (CLI binary)
- License: **Mixed.**
  - Queries: **MIT** (https://github.com/github/codeql/blob/main/LICENSE).
  - **CodeQL CLI / engine: GitHub CodeQL Terms & Conditions — proprietary,
    restricted use.** Free for academic research, demos, testing of
    OSI-licensed queries, and CI/CD on Open Source Codebases hosted on
    GitHub. Other commercial / private codebase use requires a paid GitHub
    Advanced Security license.
    Source: https://github.com/github/codeql-cli-binaries/blob/main/LICENSE.md
  → Comparison entry: "Queries MIT; CLI proprietary (GitHub CodeQL Terms)."
- Target languages: **Source-level extracted database** for C/C++, C#, Go,
  Java/Kotlin, JavaScript/TypeScript, Python, Ruby, Rust, Swift. **Not LLVM
  IR.**
- Pointer analysis: not exposed as Andersen/FS/CS — uses dataflow framework
  + type-based reasoning. **Different paradigm.**
- Value-flow / SVFG: **Not in the SVFG paradigm.** Has its own
  `DataFlow::Global` and `TaintTracking::Global` modules.
- IFDS / IDE solver: **Not exposed.** CodeQL uses a Datalog-derived
  evaluation engine on extracted databases, not IFDS.
- Taint analysis: **Yes** — `TaintTracking::Global<Config>` is the standard
  pattern. Confirmed via README example.
- Built-in checkers: **Very large library.** Hundreds of queries per
  language; thousands across all supported languages. C/C++ pack alone has
  hundreds of named queries (security, correctness, performance).
- SARIF export: **Native default.** `codeql database analyze --format=sarif-latest`
  is the canonical workflow.
- Primary SDK: **QL** (declarative query language designed for CodeQL).
- LLVM versions: **N/A.**
- Browser / WASM: **No.** VS Code extension exists, but the CLI is a native
  binary.
- Determinism: **Not advertised explicitly.**
- AI-agent / coding-agent skills: **No.**

## SAF (this project)

For completeness, the SAF column in the comparison reflects:

- License: **MIT.**
- Target IR: **LLVM IR** (`.ll` / `.bc`) and AIR-JSON.
- Pointer analysis: **Andersen CI, flow-sensitive, context-sensitive (k-CFA),
  demand-driven (DDA)** — all four variants.
- Value-flow / SVFG: **Yes.**
- IFDS / IDE solver: **Yes.**
- Taint analysis: **Yes** (source/sink/sanitizer framework with trace
  extraction).
- Built-in checkers: memory leak (CWE-401), null deref (CWE-476),
  double-free (CWE-415), use-after-free (CWE-416), taint, plus typestate
  spec slot.
- SARIF export: **Yes (native).**
- Primary SDK: **Python** (first-class authoring surface, PyO3 bindings
  over a Rust core).
- LLVM versions: **18 + 22 simultaneously** (parameterized Docker tags
  `saf-dev:llvm18` and `saf-dev:llvm22`); plan 186.
- Browser / WASM: **Yes** (playground).
- Determinism: **Yes — byte-identical outputs guaranteed (NFR-DET-001).**
- AI-agent / coding-agent skills: **Yes — 2 shipped** (saf-feature-dev,
  saf-checker-dev).

## Deep-capability verification (round 2, 2026-04-27)

After the first round, the user requested a *deeper* comparison — calling out
that SAF supports a Datalog PTA backend that SVF does not, and that there are
likely other technical capabilities worth distinguishing. Five additional
capability rows were added after this code-level investigation.

### Row: PTA solver backends

- **SAF.** Worklist solver (default, `crates/saf-analysis/src/pta/solver.rs`)
  plus a Datalog backend in `crates/saf-datalog/` powered by the Ascent
  engine. Selectable via the CLI flag `--solver datalog` (see
  `crates/saf-cli/src/help.rs`). Plan 157 documents the integration.
- **SVF.** Hand-written worklist solvers (`BVDataPTAImpl`, wave propagation,
  `Andersen.h`, `AndersenPWC.h`). No Datalog backend. WebSearch confirmed
  the published architecture is "Graph + Rules + Solver" without a
  Datalog evaluator (compare with Doop / Souffle, which are referenced
  separately).
- **Phasar.** Points-to information is computed via LLVM's basic AA + own
  call-graph algorithms (CHA/RTA/OTF). No standalone Datalog.
- **Lotus.** Multiple solvers bundled via the catalog under `lib/Alias/`:
  `InclusionBased/`, `UnificationBased/`, `DemandDriven/`. No Datalog.
- **CodeQL.** The QL query language compiles to a Datalog-style evaluation
  engine (this is QL's well-known design). So while CodeQL is not an
  LLVM-IR PTA, the evaluation paradigm is Datalog-flavored and worth
  surfacing for honest comparison.
- **Infer.** Different paradigm — Pulse is built on biabductive separation
  logic, not constraint-based PTA.

### Row: Memory SSA

- **SAF.** Hybrid MSSA (skeleton + demand-driven clobber). Verified via
  `crates/saf-analysis/src/mssa/` (`access.rs`, `builder.rs`, `modref.rs`,
  `walker.rs`).
- **SVF.** Verified by listing `svf/include/MSSA/`: `MSSAMuChi.h`,
  `MemPartition.h`, `MemRegion.h`, `MemSSA.h`, `SVFGBuilder.h`. Mature
  Memory SSA infrastructure.
- **Phasar.** Not provided — Phasar's data-flow paradigm is IFDS, not
  sparse SVFG.
- **Lotus.** Not advertised; sparse value-flow is via `DyckVFG` in
  `lib/IR/`, not a classical Memory SSA implementation.
- **CodeQL / Infer.** Different paradigms.

### Row: Numeric / abstract-interpretation domains

- **SAF.** Verified by listing `crates/saf-analysis/src/absint/`:
  `interval.rs`, `octagon/` (DBM-based), `nullness.rs`, `sccp.rs` (sparse
  conditional constant propagation), `numeric_z3.rs`. Multiple domains
  with a generic fixpoint engine (`generic_fixpoint.rs`, `fixpoint.rs`).
- **SVF.** Verified by listing `svf/include/AE/Core/`: `IntervalValue.h`,
  `NumericValue.h`, `AddressValue.h`, `RelationSolver.h`, `RelExeState.h`,
  `AbstractValue.h`, `AbstractState.h`. Abstract Execution module ships
  intervals, numeric, and relational domains.
- **Phasar.** Mono framework (`include/phasar/DataFlow/Mono/`) with
  `IntraMonoProblem.h` and `InterMonoProblem.h` — generic monotone-domain
  framework, but no canned interval / octagon implementations were
  surfaced.
- **Lotus.** Symbolic execution (`lib/Analysis/SymbolicExecution/`) plus
  Crypto / constant-time analysis (`lib/Analysis/Crypto/`, "CT-LLVM").
  Different focus from numeric AI but adjacent.
- **CodeQL.** Limited (range-analysis predicates in C/C++ pack); not a
  general numeric-domain framework.
- **Infer.** Pulse abstract domain + InferBO interval domain.

### Row: Concurrency / multi-thread analysis

- **SAF.** Verified by listing `crates/saf-analysis/src/mta/`:
  `discovery.rs`, `lockset.rs`, `mhp.rs`, `types.rs`, `export.rs`. Lockset
  + may-happen-in-parallel.
- **SVF.** Verified by listing `svf/include/MTA/`: `LockAnalysis.h`,
  `MHP.h`, `MTA.h`, `MTAStat.h`, `TCT.h` (Thread Creation Tree). Lockset +
  MHP + TCT.
- **Phasar.** Not provided.
- **Lotus.** Most extensive of the bunch. Verified by listing
  `lib/Concurrency/`: `CUDA/`, `JoinTarget/`, `LinuxKernel/`, `LockSet/`,
  `MHP/`, `MPI/`, `Memory/`, `OpenMP/`. Documentation surfaces specific
  analyses including `LockSetAnalysis`, `HappensBeforeAnalysis`,
  `MHPAnalysis`, `StaticVectorClockMHP`, `StaticThreadSharingAnalysis`,
  `EscapeAnalysis`, `JoinTargetAnalysis`, data-race detection,
  lock-mismatch detection, MPI deadlock detection.
- **CodeQL.** Some race-detection queries in the C/C++ pack but not a
  framework-level MTA.
- **Infer.** RacerD is a flagship — strong race detection, especially for
  Java.

### Row: SMT-backed reasoning / path sensitivity

- **SAF.** Verified by listing `crates/saf-analysis/src/z3_utils/`:
  `alias.rs`, `assertions.rs`, `condition_prover.rs`, `dominator.rs`,
  `interprocedural.rs`, `reachability.rs`, `solver.rs`. Plus
  `crates/saf-analysis/src/checkers/pathsens*.rs` (path-sensitive checker
  runner) and `crates/saf-analysis/src/checkers/z3solver.rs`. Joint
  feasibility, condition-proof, alias refinement via Z3.
- **SVF.** Path-sensitive analysis is provided through Saber's
  `SrcSnkSolver` / `SaberCondAllocator`, but not a general Z3-backed SMT
  framework — partial.
- **Phasar.** `include/phasar/DataFlow/PathSensitivity/` ships
  `ExplodedSuperGraph.h`, `FlowPath.h`, `PathSensitivityManager*.h`,
  `PathTracingFilter.h`. Path-sensitive infrastructure exists; SMT use
  not directly evidenced from the include listing.
- **Lotus.** SMT solvers under `lib/Solvers/SMT/`, plus EGraph; KINT
  uses SMT for integer-bug detection; SymbolicExecution module exists in
  `lib/Analysis/SymbolicExecution/`.
- **CodeQL.** No SMT — CodeQL is Datalog-style evaluation only.
- **Infer.** Pulse's biabduction step uses SMT for satisfiability.

## Deep-capability verification (round 3, 2026-04-27)

Three more capability rows added: custom checker authoring, interactive graph
query API, and specialized data structures. The user surfaced each as a real
SAF differentiator and asked for a fair comparison.

### Row: Custom checker authoring

- **SAF.** Declarative YAML specs in `share/saf/specs/` (taint sources,
  sinks, sanitizers; libc / posix / gnu / cxxabi function specs).
  Reachability modes: `may_reach`, `must_not_reach`, `multi_reach`,
  `never_reach_sink` (CLAUDE.md "Key Decisions" section). A new checker
  is typically YAML, not Rust code.
- **SVF.** C++ subclassing — a new bug-finder extends Saber's
  `SrcSnkDDA` / `SrcSnkSolver` and overrides hooks. Verified by listing
  `svf/include/SABER/`.
- **Phasar.** C++ subclassing — `IFDSTabulationProblem` /
  `IDETabulationProblem`. Verified at
  `include/phasar/DataFlow/IfdsIde/`.
- **Lotus.** C++ checker plug-ins under `lib/Checker/<area>/`.
- **CodeQL.** A QL query *is* a checker. The language and the checker
  authoring surface are the same thing.
- **Infer.** OCaml plug-ins via the Infer.AI abstract-domain framework
  (https://fbinfer.com/docs/absint-framework/) — define a domain,
  transfer functions, and the framework runs the analysis across
  supported languages.

### Row: Interactive graph query API

- **SAF.** Verified by listing `crates/saf-python/src/`: `combined.rs`,
  `cspta.rs`, `dda.rs`, `cg_refinement.rs`, etc. each expose `points_to`
  / `points_to_in_context` / `call_graph_export` PyO3 methods. The
  README's Python example uses `proj.query()` → `q.taint_flow(...)` /
  `q.points_to(...)`. CLI mirrors this via `saf query points-to / cfg /
  callgraph` (verified in `crates/saf-cli/src/help.rs`).
- **SVF.** Programmatic access to ICFG / SVFG / CG / VFG via the C++
  API. There is no separate user-facing query DSL — the same C++
  classes are used to author checkers and to traverse graphs.
- **Phasar.** Solver results (`SolverResults.h` in `IfdsIde/`) are
  accessible programmatically in C++, but again, no separate query DSL.
- **Lotus.** C++ API exposes the bundled-backend graphs.
- **CodeQL.** The QL language is itself the graph query API. Every
  capability — including checkers — is expressed as a query.
- **Infer.** Surfaces findings, not graph queries. There is no exposed
  graph query interface; users interact via `infer run --check ...` and
  the resulting reports.

### Row: Specialized data structures (perf)

- **SAF.** Verified by listing `crates/saf-analysis/src/pta/ptsset/`:
  `roaring_pts.rs` (Roaring bitmaps — default for ≥10K allocation
  sites), `fxhash.rs` (FxHash for smaller sets), `btree.rs`, `bdd.rs`,
  `id_bitset.rs`, plus `indexer.rs` for the frozen indexer that enables
  lock-free solving. `config.rs` chooses between them per workload.
  Crate-level deps confirm `roaring`, `rustc-hash` (FxHash), `indexmap`,
  `rayon` (parallelism), `smallvec`. Logged in CLAUDE.md "Key Decisions":
  "PtsSet | Roaring default (≥10K alloc sites), FxHash (<10K), frozen
  indexer for lock-free solving."
- **SVF.** Bit vectors — `BVDataPTAImpl` is the base class for both
  flow-sensitive and flow-insensitive PTA, mapping pointers to
  bit-vector points-to sets (verified via SVF wiki Technical
  Documentation).
- **Phasar.** `EdgeFunctionSingletonCache` (verified in
  `include/phasar/DataFlow/IfdsIde/DefaultEdgeFunctionSingletonCache.h`
  and `EdgeFunctionSingletonCache.h`) memoizes IFDS/IDE edge functions;
  Phasar additionally uses small-object-optimization for short edge
  functions (per ECOOP 2024 paper "Scaling Interprocedural Static
  Data-Flow Analysis to Large C/C++ Applications").
- **Lotus.** Varies by bundled backend; not advertised at the
  framework level.
- **CodeQL.** Datalog evaluation engine with BDDs (Binary Decision
  Diagrams) for some predicates, plus special data structures for
  transitive closures. Per the CodeQL docs: "BDD operations take time
  proportional to the size of the data structure, not the number of
  tuples in a relation, which leads to fast execution times."
- **Infer.** Bi-abductive function summaries computed compositionally,
  with a persistent summary cache that supports incremental analysis on
  code modifications (per the Communications of the ACM article
  "Scaling Static Analyses at Facebook").

## Caveats to acknowledge in the comparison page

1. **Comparing apples and oranges.** SVF, Phasar, Lotus, and SAF are all
   LLVM-IR-based frameworks. CodeQL and Infer are source-level / AST-level.
   The comparison page should explicitly note this and explain why CodeQL
   and Infer are still listed (because users *do* compare across these for
   "should I pick a static analyzer?" decisions).
2. **Primary SDK ≠ exclusive SDK.** SVF gained `Pysvf` recently; Phasar may
   add bindings. The comparison shows the *primary* authoring surface and
   notes secondary bindings.
3. **Multi-LLVM "support" semantics.** SAF ships *two simultaneous* tags;
   SVF supports many LLVM versions on different historical branches but
   only one per build at HEAD. The comparison should distinguish
   *simultaneous* multi-LLVM from *historical* multi-LLVM.
4. **Determinism.** "Byte-identical output" is a strong claim. Most peers
   probably *are* deterministic in practice but don't make it a contractual
   guarantee. The comparison should phrase this carefully:
   "byte-identical guarantee: SAF — yes (contractual); others — likely
   in practice, not a guaranteed contract."
5. **Built-in checker counts** are not directly comparable. CodeQL has
   hundreds of queries because it covers many languages. Infer has many
   because it covers many bug classes per language. SVF has few because
   the framework focuses on the analyses, not the checkers. The comparison
   page should note this with a sentence each.
