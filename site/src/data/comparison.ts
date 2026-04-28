// Tool comparison data — backs both the homepage teaser and the /comparison/ page.
// Every claim here is sourced in plans/189-research-notes.md.
// When peer tools change, update both files together.

export type ToolKey = 'saf' | 'svf' | 'phasar' | 'lotus' | 'codeql' | 'infer';

export type CellKind = 'yes' | 'no' | 'partial' | 'na' | 'plain';

export interface CellValue {
  text: string;
  kind: CellKind;
}

export interface ComparisonRow {
  dimension: string;
  group: 'capability' | 'differentiation';
  /** Whether to include this row in the homepage teaser. */
  teaser: boolean;
  /** Optional explanatory note shown on the dedicated /comparison/ page. */
  note?: string;
  values: Record<ToolKey, CellValue>;
}

export interface ToolMeta {
  key: ToolKey;
  name: string;
  url: string;
  /** "Best at" prose, 1-2 sentences. Shown below the table. */
  bestAt: string;
}

export const TOOLS: ToolMeta[] = [
  {
    key: 'saf',
    name: 'SAF',
    url: 'https://github.com/Static-Analyzer-Factory/static-analyzer-factory',
    bestAt:
      'An extensible program-analysis framework for LLVM IR, with a Python-first SDK over a Rust core. Best for researchers and engineers who want a modern, deterministic platform with multi-LLVM support, low-friction tooling, and a browser playground for sharing work.',
  },
  {
    key: 'svf',
    name: 'SVF',
    url: 'https://github.com/SVF-tools/SVF',
    bestAt:
      'The most mature LLVM-IR value-flow framework, with a broad pointer-analysis catalog and a strong publication record. Best when you need state-of-the-art SVFG-based analyses out of the box.',
  },
  {
    key: 'phasar',
    name: 'Phasar',
    url: 'https://github.com/secure-software-engineering/phasar',
    bestAt:
      'The reference IFDS/IDE solver for LLVM IR. Best when your problem fits the IFDS or IDE model and you want a clean C++ API for declaring data-flow problems.',
  },
  {
    key: 'lotus',
    name: 'Lotus',
    url: 'https://github.com/ZJU-PL/lotus',
    bestAt:
      'An umbrella research framework that bundles many alias analyses (DyckAA, SparrowAA, Sea-DSA, AserPTA, AllocAA), checkers (FiTx, KINT, Saber, Pulse, Security), and abstract-interpretation engines. Best for academics comparing multiple alias analyses on the same input.',
  },
  {
    key: 'codeql',
    name: 'CodeQL',
    url: 'https://github.com/github/codeql',
    bestAt:
      "GitHub's semantic code analysis engine, with a vast query library across 10+ languages and native SARIF output. Best for production security teams already invested in GitHub Advanced Security — but note the CLI is proprietary outside open-source use.",
  },
  {
    key: 'infer',
    name: 'Infer',
    url: 'https://github.com/facebook/infer',
    bestAt:
      "Meta's abstract-interpretation-based analyzer for C/C++/Obj-C/Java, with deep checker libraries (Pulse, RacerD, Quandary). Best for industrial codebases where source-level analysis without LLVM IR is preferred.",
  },
];

const yes = (text = 'Yes'): CellValue => ({ text, kind: 'yes' });
const no = (text = 'No'): CellValue => ({ text, kind: 'no' });
const partial = (text: string): CellValue => ({ text, kind: 'partial' });
const na = (text = 'N/A'): CellValue => ({ text, kind: 'na' });
const plain = (text: string): CellValue => ({ text, kind: 'plain' });

export const ROWS: ComparisonRow[] = [
  // ---------------- Capability rows ----------------
  {
    dimension: 'Primary IR / target',
    group: 'capability',
    teaser: false,
    values: {
      saf: plain('LLVM IR (C/C++)'),
      svf: plain('LLVM IR (C/C++)'),
      phasar: plain('LLVM IR (C/C++)'),
      lotus: plain('LLVM IR (C/C++)'),
      codeql: plain('Source DB (10+ langs)'),
      infer: plain('Source AST (C/C++/ObjC/Java)'),
    },
  },
  {
    dimension: 'Pointer analysis variants',
    group: 'capability',
    teaser: false,
    note: 'CI = context-insensitive, FS = flow-sensitive, CS = context-sensitive (k-CFA), DDA = demand-driven. Tools without an exposed taxonomy are listed by their actual approach.',
    values: {
      saf: yes('CI, FS, CS, DDA'),
      svf: yes('CI, FS, CS, DDA'),
      phasar: partial('Computed internally (CG + AA)'),
      lotus: yes('Many bundled (DyckAA, SparrowAA, Sea-DSA, …)'),
      codeql: partial('Different paradigm'),
      infer: partial('Different paradigm'),
    },
  },
  {
    dimension: 'PTA solver backends',
    group: 'capability',
    teaser: false,
    note: 'How the pointer-analysis solver is implemented. SAF ships both a worklist solver (default) and a Datalog backend (opt-in via --solver datalog, powered by the Ascent engine). SVF uses hand-written worklist solvers. Phasar computes points-to information through LLVM AA + its own call-graph algorithms. Lotus combines inclusion-based, unification (DyckAA), and demand-driven solvers. CodeQL uses Datalog-style evaluation as its underlying QL engine. Infer uses separation logic.',
    values: {
      saf: yes('Worklist + Datalog (Ascent)'),
      svf: plain('Worklist (wave + bit-vector)'),
      phasar: plain('LLVM AA + own CG algorithms'),
      lotus: plain('Inclusion + unification + DDA'),
      codeql: yes('QL → Datalog evaluation'),
      infer: plain('Separation logic'),
    },
  },
  {
    dimension: 'Value-flow / SVFG',
    group: 'capability',
    teaser: false,
    values: {
      saf: yes(),
      svf: yes('Headline feature'),
      phasar: no('IFDS-based'),
      lotus: yes('DyckVFG variant'),
      codeql: partial('DataFlow module'),
      infer: no(),
    },
  },
  {
    dimension: 'Memory SSA',
    group: 'capability',
    teaser: false,
    note: 'Sparse value-flow infrastructure used to scale flow-sensitive analyses. Phasar takes a different route via IFDS; Lotus expresses sparse value-flow through DyckVFG; CodeQL and Infer use different paradigms entirely.',
    values: {
      saf: yes('Hybrid (skeleton + demand-driven)'),
      svf: yes('MemSSA + MemRegion'),
      phasar: no('IFDS-based instead'),
      lotus: partial('DyckVFG instead'),
      codeql: no('Different paradigm'),
      infer: no('Different paradigm'),
    },
  },
  {
    dimension: 'IFDS / IDE solver',
    group: 'capability',
    teaser: false,
    values: {
      saf: yes(),
      svf: no(),
      phasar: yes('Specialty'),
      lotus: partial('Bundles Phasar'),
      codeql: no(),
      infer: no(),
    },
  },
  {
    dimension: 'Taint analysis',
    group: 'capability',
    teaser: false,
    note: 'A first-class taint framework with sources, sinks, and sanitizers. SVF supports taint patterns by composing on top of its SVFG; Lotus surfaces taint within KINT (integer-bug detection).',
    values: {
      saf: yes('Source/sink/sanitizer'),
      svf: partial('Built on SVFG'),
      phasar: yes('IFDS + IDE taint clients'),
      lotus: partial('Within KINT'),
      codeql: yes('TaintTracking::Global'),
      infer: yes('Quandary + Pulse'),
    },
  },
  {
    dimension: 'Numeric / abstract domains',
    group: 'capability',
    teaser: false,
    note: 'Abstract-interpretation domains shipped for numeric/value reasoning beyond pointer aliasing.',
    values: {
      saf: yes('Intervals, octagons, nullness, SCCP'),
      svf: yes('Intervals, numeric, relational (AE)'),
      phasar: yes('Monotone framework (intra + inter)'),
      lotus: yes('Symbolic execution + constant-time analysis'),
      codeql: partial('Range analysis (built into queries)'),
      infer: yes('Pulse abstract domain + InferBO intervals'),
    },
  },
  {
    dimension: 'Concurrency / MTA',
    group: 'capability',
    teaser: false,
    note: 'Multi-thread analysis: lockset, may-happen-in-parallel, race detection. Lotus has the broadest scope here, with bundled support for kernel, MPI, OpenMP, and CUDA models.',
    values: {
      saf: yes('Lockset + MHP'),
      svf: yes('LockAnalysis, MHP, TCT'),
      phasar: no('Out of scope'),
      lotus: yes('Race, MPI, OpenMP, CUDA, kernel'),
      codeql: partial('Race-detection queries'),
      infer: yes('RacerD'),
    },
  },
  {
    dimension: 'SMT-backed reasoning',
    group: 'capability',
    teaser: false,
    note: 'SMT-solver integration for path conditions, joint feasibility, and refinement.',
    values: {
      saf: yes('Z3 (conditions, reachability, alias)'),
      svf: partial('Saber path-sensitive solver'),
      phasar: yes('PathSensitivity module'),
      lotus: yes('SMT solvers + KINT + symbolic execution'),
      codeql: no('Datalog evaluation, no SMT'),
      infer: yes('Pulse uses SMT'),
    },
  },
  {
    dimension: 'Built-in checker library',
    group: 'capability',
    teaser: false,
    note: 'Counts are not directly comparable: CodeQL is large because it spans many languages; Infer covers many bug classes per language; SVF, Phasar, and Lotus are framework-first but each still ships a meaningful catalog. Verified by file-listing the corresponding source directories — see plans/189-research-notes.md.',
    values: {
      saf: plain('5+ (CWE-401/415/416/476, taint)'),
      svf: plain('3 SABER (leak, double-free, file)'),
      phasar: plain('15+ IFDS/IDE clients'),
      lotus: plain('Many (FiTx, KINT, Saber, Concurrency, Security)'),
      codeql: plain('Hundreds (per language)'),
      infer: plain('30+ (Pulse, RacerD, …)'),
    },
  },
  {
    dimension: 'Custom checker authoring',
    group: 'capability',
    teaser: false,
    note: 'How you write a new bug-finding checker. SAF uses declarative YAML specs with modes like may_reach / must_not_reach / multi_reach, so a typical checker is configuration rather than code. SVF, Phasar, Lotus, and Infer require subclassing or plug-in registration in their host language. CodeQL is query-first — every checker is a QL query.',
    values: {
      saf: yes('Declarative YAML specs (may_reach modes)'),
      svf: plain('C++ subclassing (Saber framework)'),
      phasar: plain('C++ IFDS/IDE problem subclasses'),
      lotus: plain('C++ checker plug-ins'),
      codeql: yes('QL queries (the language is the checker)'),
      infer: plain('OCaml + Infer.AI abstract domain'),
    },
  },
  {
    dimension: 'Interactive graph query API',
    group: 'capability',
    teaser: false,
    note: 'Whether the framework exposes a separate API for interactively asking questions about analysis graphs (CFG, callgraph, points-to, value-flow) without writing a full checker. SAF ships a Python SDK and CLI commands for points-to, taint-flow, flows, callgraph, and CFG queries. CodeQL\'s entire model is graph queries written in QL. SVF, Phasar, and Lotus expose graph data through their C++ API but it is the same surface used to build checkers. Infer surfaces results, not graph queries.',
    values: {
      saf: yes('Python SDK + CLI (points-to, flows, taint)'),
      svf: partial('C++ API (raw graph traversal)'),
      phasar: partial('C++ API (solver results)'),
      lotus: partial('C++ API (raw)'),
      codeql: yes('QL (queries are the model)'),
      infer: no('Results only'),
    },
  },
  {
    dimension: 'Specialized data structures (perf)',
    group: 'capability',
    teaser: false,
    note: 'Memory-/time-efficient data structures used in the analysis core. SAF ships a polymorphic PtsSet selectable per workload — Roaring bitmaps by default for ≥10K allocation sites, FxHash for smaller sets, plus BTree / BDD / id_bitset variants — alongside a frozen indexer for lock-free solving and Rayon-based parallelism. SVF uses bit-vector points-to sets (BVDataPTAImpl). Phasar uses an EdgeFunctionSingletonCache with small-object-optimization for IFDS/IDE edge functions. CodeQL relies on BDDs and Datalog relational-algebra indexes. Infer compresses analysis state via bi-abductive function summaries with a persistent summary cache.',
    values: {
      saf: yes('Roaring + FxHash + frozen indexer + Rayon'),
      svf: plain('Bit-vector points-to (BVDataPTAImpl)'),
      phasar: plain('EdgeFunctionSingletonCache + SOO'),
      lotus: plain('Varies by bundled backend'),
      codeql: yes('BDDs + Datalog indexes'),
      infer: yes('Bi-abductive summaries + cache'),
    },
  },
  {
    dimension: 'SARIF export',
    group: 'capability',
    teaser: false,
    values: {
      saf: yes('Native'),
      svf: no(),
      phasar: no(),
      lotus: no(),
      codeql: yes('Native (default)'),
      infer: partial('External adapter'),
    },
  },
  {
    dimension: 'License',
    group: 'capability',
    teaser: false,
    values: {
      saf: plain('MIT'),
      svf: plain('AGPL-3.0'),
      phasar: plain('MIT'),
      lotus: plain('MIT (mixed deps)'),
      codeql: plain('Queries MIT; CLI proprietary'),
      infer: plain('MIT'),
    },
  },

  // ---------------- Differentiation rows ----------------
  {
    dimension: 'Primary SDK / authoring language',
    group: 'differentiation',
    teaser: true,
    values: {
      saf: yes('Python (Rust core)'),
      svf: plain('C++ (Pysvf wrapper)'),
      phasar: plain('C++ (C++20)'),
      lotus: plain('C++'),
      codeql: plain('QL (DSL)'),
      infer: plain('OCaml'),
    },
  },
  {
    dimension: 'Multi-LLVM version (simultaneous)',
    group: 'differentiation',
    teaser: true,
    note: 'Distinct from "supports historical LLVM versions on different branches" (which SVF and others do). We mean: ships multiple ready-to-run builds for different LLVM toolchains at the same time, side by side.',
    values: {
      saf: yes('LLVM 18 + 22'),
      svf: no('One per build (broad history)'),
      phasar: yes('LLVM 16 + 17'),
      lotus: no('LLVM 14'),
      codeql: na(),
      infer: na(),
    },
  },
  {
    dimension: 'Browser / WASM playground',
    group: 'differentiation',
    teaser: true,
    values: {
      saf: yes('Pyodide + WASM'),
      svf: no(),
      phasar: no(),
      lotus: no(),
      codeql: no(),
      infer: no(),
    },
  },
  {
    dimension: 'Byte-deterministic output',
    group: 'differentiation',
    teaser: true,
    note: 'All listed peers are deterministic in practice; SAF makes byte-identical output a contractual non-functional requirement (NFR-DET-001), which is the distinction we surface here.',
    values: {
      saf: yes('Contractual (NFR-DET-001)'),
      svf: partial('Not advertised as contract'),
      phasar: partial('Not advertised as contract'),
      lotus: partial('Not advertised as contract'),
      codeql: partial('Not advertised as contract'),
      infer: partial('Not advertised as contract'),
    },
  },
  {
    dimension: 'AI-agent / coding-agent skills',
    group: 'differentiation',
    teaser: true,
    note: 'Skills are pre-built workflows that guide AI coding agents (Claude Code, Codex, etc.) through tool-specific tasks like adding analyses or authoring checkers.',
    values: {
      saf: yes('2 (feature-dev, checker-dev)'),
      svf: no(),
      phasar: no(),
      lotus: partial('AGENTS.md present'),
      codeql: no(),
      infer: no(),
    },
  },
];

export const TEASER_ROWS = ROWS.filter((r) => r.teaser);
