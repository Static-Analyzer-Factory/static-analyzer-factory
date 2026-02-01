// Built-in help system for the SAF CLI (FR-CLI-003).
// Provides topic-based guides accessible via `saf help [topic]`.

/// Display a help topic guide, or the overview if no topic is given.
pub fn print_help(topic: Option<&str>) -> anyhow::Result<()> {
    match topic {
        None => print_overview(),
        Some("run") => print_run_guide(),
        Some("checkers") => print_checkers_guide(),
        Some("pta") => print_pta_guide(),
        Some("typestate") => print_typestate_guide(),
        Some("taint") => print_taint_guide(),
        Some("z3") => print_z3_guide(),
        Some("export") => print_export_guide(),
        Some("specs") => print_specs_guide(),
        Some("incremental") => print_incremental_guide(),
        Some("examples") => print_examples(),
        Some(other) => {
            anyhow::bail!("Unknown help topic: '{other}'\n\nRun 'saf help' for available topics.")
        }
    }
    Ok(())
}

fn print_overview() {
    println!(
        "\
SAF -- Static Analyzer Factory

COMMANDS
  run           Run analysis passes on input programs
  index         Index input files via a frontend to produce AIR
  export        Export graphs or findings (JSON, SARIF, DOT, HTML)
  query         Query analysis results (points-to, flows, taint)
  schema        Print the SAF schema (frontends, queries, config)
  specs         Manage function specifications (list, validate, lookup)
  incremental   Run incremental analysis with caching
  help          Show this overview or a topic guide

HELP TOPICS
  saf help run           How to run analyses and configure passes
  saf help checkers      Built-in bug checkers and how to select them
  saf help pta           Pointer analysis variants and configuration
  saf help typestate     Typestate checking and custom protocol specs
  saf help taint         Taint analysis modes and configuration
  saf help z3            Z3-based path sensitivity and refinement
  saf help export        Export targets, formats, and examples
  saf help specs         Function specification format and discovery
  saf help incremental   Incremental analysis and caching
  saf help examples      Common usage patterns and recipes"
    );
}

fn print_run_guide() {
    println!(
        "\
SAF HELP: run

SYNOPSIS
  saf run [OPTIONS] <input>

DESCRIPTION
  The 'run' command ingests an LLVM IR file (.ll or .bc), builds
  internal representations (CFG, call graph, pointer analysis),
  and runs the selected checkers to produce findings.

BASIC USAGE
  saf run program.ll                      Run with default settings
  saf run program.ll --mode fast          Fast mode (fewer iterations)
  saf run program.ll --mode precise       Precise mode (full fixed-point)

CHECKER SELECTION
  --checkers all                          Enable all checkers (default)
  --checkers memory-leak,null-deref       Enable specific checkers
  --checkers numeric                      Enable numeric checkers only
  --checkers none                         Disable all checkers

POINTER ANALYSIS
  --pta andersen                          Flow-insensitive (default)
  --pta cspta --pta-k 2                   Context-sensitive k-CFA
  --pta fspta                             Flow-sensitive
  --pta dda                               Demand-driven

OUTPUT
  --format json                           JSON output (default)
  --format sarif                          SARIF for CI integration
  --output report.json                    Write to file instead of stdout
  --serve                                 Start JSON protocol server

FRONTEND
  --frontend llvm                         LLVM IR frontend (default)
  --frontend air-json                     AIR-JSON frontend

See also: saf help checkers, saf help pta, saf help examples"
    );
}

fn print_checkers_guide() {
    println!(
        "\
SAF HELP: checkers

BUILT-IN SVFG CHECKERS
  These checkers operate on the Sparse Value-Flow Graph (SVFG) and
  track resource lifecycles through the program.

  Name                    CWE      Description
  ----                    ---      -----------
  memory-leak             CWE-401  Detects unreleased heap allocations
  use-after-free          CWE-416  Detects dereference of freed heap memory
  double-free             CWE-415  Detects double-free of heap memory
  null-deref              CWE-476  Detects null pointer dereference
  file-descriptor-leak    CWE-775  Detects unclosed file descriptors
  uninit-use              CWE-908  Detects use of uninitialized memory
  stack-escape            CWE-562  Detects stack pointer escaping function scope
  lock-not-released       CWE-764  Detects unreleased mutex locks
  generic-resource-leak   CWE-772  Detects unreleased generic resources

NUMERIC CHECKERS
  These checkers use abstract interpretation over numeric domains.

  Name                    CWE      Description
  ----                    ---      -----------
  buffer_overflow         CWE-120  Detects array access beyond bounds
  integer_overflow        CWE-190  Detects arithmetic overflow/underflow
  division_by_zero        CWE-369  Detects potential division by zero

SELECTING CHECKERS
  saf run program.ll --checkers all                 All checkers (default)
  saf run program.ll --checkers memory-leak,null-deref   Specific checkers
  saf run program.ll --checkers numeric             Numeric checkers only
  saf run program.ll --checkers none                Disable all checkers

See also: saf help run, saf help typestate"
    );
}

fn print_pta_guide() {
    println!(
        "\
SAF HELP: pta (Pointer Analysis)

ANALYSIS VARIANTS
  Andersen (default)
    Flow-insensitive, context-insensitive inclusion-based analysis.
    Fast and scales to large programs. Good for most use cases.
    Usage: saf run program.ll --pta andersen

  CSPTA (Context-Sensitive)
    Context-sensitive k-CFA analysis. More precise than Andersen,
    distinguishes calling contexts up to depth k.
    Usage: saf run program.ll --pta cspta --pta-k 2

  FSPTA (Flow-Sensitive)
    Flow-sensitive analysis that tracks pointer state changes along
    execution paths. More precise for programs with complex control flow.
    Usage: saf run program.ll --pta fspta

  DDA (Demand-Driven)
    Only computes points-to information for queried pointers.
    Efficient when only a subset of results is needed.
    Usage: saf run program.ll --pta dda

SOLVER BACKENDS
  --solver worklist       Incremental worklist solver (default)
  --solver datalog        Batch Datalog solver (Ascent engine)

POINTS-TO SET REPRESENTATIONS
  --pts auto              Automatic selection based on program size (default)
  --pts btreeset          Ordered BTreeSet (deterministic)
  --pts fxhash            Hash-based (fast, non-deterministic iteration)
  --pts roaring           Roaring bitmaps (compact for large sets)
  --pts bdd               Binary decision diagrams (for very large programs)

FIELD SENSITIVITY
  --field-sensitivity struct-fields    Track struct fields (default)
  --field-sensitivity array-index      Track array indices
  --field-sensitivity flat             No field tracking (fastest)

See also: saf help run, saf help z3"
    );
}

fn print_typestate_guide() {
    println!(
        "\
SAF HELP: typestate

OVERVIEW
  Typestate analysis tracks the protocol state of resources through
  a program. Each resource (file handle, socket, lock, etc.) has a
  finite state machine defining valid operation sequences. SAF reports
  violations where operations occur in invalid states.

  Example: a file must be opened before reading and closed after use.
  The typestate checker verifies this protocol is followed on all paths.

BUILT-IN TYPESTATE SPECS
  file-io          File open/read/write/close protocol
  socket           Socket create/bind/listen/accept/close protocol
  mutex            Lock acquire/release protocol
  memory           Allocation/use/free protocol

  saf run program.ll --typestate file-io
  saf run program.ll --typestate mutex

CUSTOM TYPESTATE SPECS
  Define custom protocols in YAML and pass them to SAF:

  saf run program.ll --typestate-custom my-spec.yaml

  YAML format:
    name: my-protocol
    states: [Init, Open, Closed]
    initial: Init
    error_states: [Closed]
    transitions:
      - from: Init
        call: api_open
        to: Open
      - from: Open
        call: api_close
        to: Closed
    violations:
      - state: Init
        call: api_use
        message: \"resource used before opening\"

See also: saf help checkers, saf help specs"
    );
}

fn print_taint_guide() {
    println!(
        "\
SAF HELP: taint

OVERVIEW
  Taint analysis tracks the flow of untrusted data from sources
  (e.g., user input) to sinks (e.g., system calls) to detect
  injection vulnerabilities.

  SAF provides two taint analysis modes:

VALUE-FLOW TAINT
  Tracks taint along value-flow edges in the SVFG. Good for
  precise, on-demand queries between specific source-sink pairs.

  saf query taint --input program.ll <source> <sink>

  Where <source> and <sink> are value IDs or function names.

IFDS TAINT
  Uses the IFDS (Interprocedural Finite Distributive Subset)
  framework for whole-program taint analysis. Configured via
  a YAML file specifying sources, sinks, and sanitizers.

  saf run program.ll --ifds-taint config.yaml

  YAML format:
    sources:
      - function: getenv
        param: return
      - function: read
        param: 1
    sinks:
      - function: system
        param: 0
      - function: execve
        param: 0
    sanitizers:
      - function: sanitize_input
        param: 0

See also: saf help run, saf help checkers"
    );
}

fn print_z3_guide() {
    println!(
        "\
SAF HELP: z3

OVERVIEW
  SAF integrates with Z3 for path-sensitive analysis. Z3 can filter
  false positives by checking path feasibility, prove assertions,
  and refine alias analysis results.

OPTIONS
  --path-sensitive
    Filter false-positive findings via path-feasibility checks.
    For each reported finding, Z3 checks whether the path leading
    to the bug is actually feasible. Infeasible paths are removed.

    saf run program.ll --path-sensitive

  --z3-prove
    Use Z3 to prove user assertions in the program. Reports
    whether each assertion holds, fails, or is unknown.

    saf run program.ll --z3-prove

  --z3-refine-alias
    Refine alias analysis results using Z3 constraint solving.
    Can disprove spurious alias pairs that pointer analysis
    reports conservatively.

    saf run program.ll --z3-refine-alias

  --z3-check-reachability
    Check whether specific program points are reachable given
    path constraints.

    saf run program.ll --z3-check-reachability

  --z3-timeout <ms>
    Set the Z3 solver timeout in milliseconds. Default: 5000ms.
    Increase for complex programs; decrease for faster analysis.

    saf run program.ll --path-sensitive --z3-timeout 10000

COMBINING OPTIONS
  Z3 options can be combined:

  saf run program.ll --path-sensitive --z3-refine-alias --z3-timeout 8000

See also: saf help run, saf help pta"
    );
}

fn print_export_guide() {
    println!(
        "\
SAF HELP: export

SYNOPSIS
  saf export <target> --input <file> [--format <fmt>] [--output <path>]

EXPORT TARGETS
  cfg          Control-flow graph
  callgraph    Call graph (inter-procedural)
  defuse       Def-use chains
  valueflow    Value-flow graph (SVFG)
  svfg         Sparse value-flow graph
  findings     Analysis findings / bug reports
  pta          Points-to analysis results

OUTPUT FORMATS
  json         JSON PropertyGraph format (default)
  sarif        SARIF 2.1.0 (findings only, for CI integration)
  dot          Graphviz DOT (graph targets only)
  html         Interactive HTML visualization

EXAMPLES
  saf export callgraph --input program.ll
  saf export callgraph --input program.ll --format dot --output cg.dot
  saf export findings --input program.ll --format sarif --output report.sarif
  saf export cfg --input program.ll --format html --output cfg.html
  saf export valueflow --input program.ll --format json --output vfg.json

PROPERTY GRAPH FORMAT
  All graph exports use a unified PropertyGraph JSON format:

  {{
    \"schema_version\": \"0.1.0\",
    \"graph_type\": \"<type>\",
    \"metadata\": {{}},
    \"nodes\": [{{\"id\": \"0x...\", \"labels\": [...], \"properties\": {{...}}}}],
    \"edges\": [{{\"src\": \"0x...\", \"dst\": \"0x...\", \"edge_type\": \"...\"}}]
  }}

See also: saf help run, saf help examples"
    );
}

fn print_specs_guide() {
    println!(
        "\
SAF HELP: specs

OVERVIEW
  Function specifications tell SAF how external and library functions
  behave (allocation, deallocation, taint propagation, etc.) without
  analyzing their implementation. Specs are defined in YAML files.

SPEC DISCOVERY PATHS (in priority order)
  1. <binary>/../share/saf/specs/*.yaml   Shipped default specs
  2. ~/.saf/specs/*.yaml                  User global specs
  3. ./saf-specs/*.yaml                   Project local specs
  4. $SAF_SPECS_PATH/*.yaml               Explicit override

  Later paths override earlier ones for the same function name.

MANAGING SPECS
  saf specs list                 List all loaded specs
  saf specs list --verbose       Show detailed spec information
  saf specs validate <path>      Validate spec file(s) at path
  saf specs lookup <name>        Look up the spec for a function

YAML SPEC FORMAT
  specs:
    - name: malloc
      role: Allocator
      params:
        - index: 0
          semantic: AllocSize
      returns:
        semantic: AllocResult
        points_to: heap

    - name: free
      role: Deallocator
      params:
        - index: 0
          semantic: DeallocTarget

    - name: strlen
      pure: true
      params:
        - index: 0
          semantic: ReadOnly
      returns:
        semantic: StringLength

See also: saf help checkers, saf help typestate"
    );
}

fn print_incremental_guide() {
    println!(
        "\
SAF HELP: incremental

OVERVIEW
  Incremental analysis caches results and only recomputes what
  changed between runs. This dramatically speeds up repeated
  analysis during development.

BASIC USAGE
  saf incremental src/*.ll                 Analyze with default cache
  saf incremental src/*.ll --cache-dir .saf-cache

CACHE DIRECTORY
  --cache-dir <path>    Set cache location (default: .saf-cache)

  The cache stores:
  - File fingerprints (BLAKE3 hashes for change detection)
  - Function summaries (pre-computed analysis results)
  - Constraint caches (PTA constraints per module)

CHANGE DETECTION
  SAF uses BLAKE3 fingerprints to detect which input files changed.
  Only modules with changed fingerprints are re-analyzed. Dependent
  modules are transitively invalidated based on the call graph.

DRY RUN (PLAN)
  --plan                Show what would be recomputed without running

  saf incremental src/*.ll --plan

  This shows added, removed, and changed modules along with the
  recomputation steps that would be performed.

CACHE MANAGEMENT
  --clean               Clear the cache before analysis

  saf incremental src/*.ll --clean

SUMMARY EXPORT
  --export-summaries <path>    Export cached summaries as YAML

  saf incremental src/*.ll --export-summaries summaries.yaml

  Exported summaries can be used as function specs for downstream
  analysis, enabling modular and compositional workflows.

See also: saf help run, saf help specs"
    );
}

fn print_examples() {
    println!(
        "\
SAF HELP: examples

QUICK SCAN
  saf run program.ll

  Runs all checkers with default settings (Andersen PTA, precise mode).

CI PIPELINE WITH SARIF OUTPUT
  saf run program.ll --format sarif --output report.sarif

  Produces SARIF 2.1.0 output for integration with GitHub Code Scanning,
  VS Code SARIF Viewer, and other SARIF-compatible tools.

TAINT ANALYSIS
  saf run program.ll --ifds-taint taint-config.yaml

  Whole-program taint analysis with IFDS. The YAML file specifies
  sources (e.g., getenv, read), sinks (e.g., system, execve), and
  optional sanitizers.

CUSTOM TYPESTATE CHECKER
  saf run program.ll --typestate-custom api-spec.yaml

  Run a custom typestate protocol checker defined in YAML.

EXPORT CALL GRAPH AS DOT
  saf export callgraph --input program.ll --format dot --output cg.dot
  dot -Tpng cg.dot -o cg.png

  Export the call graph in Graphviz DOT format and render to PNG.

INCREMENTAL ANALYSIS
  saf incremental src/*.ll --cache-dir .saf-cache

  Analyze all files with caching. Subsequent runs only re-analyze
  changed files and their dependents.

DEMAND-DRIVEN POINTS-TO QUERY
  saf query points-to --input program.ll --value %ptr

  Query what a specific pointer may point to using demand-driven PTA.

PATH-SENSITIVE ANALYSIS
  saf run program.ll --path-sensitive --z3-timeout 10000

  Run all checkers with Z3-based path feasibility filtering.
  Reduces false positives at the cost of analysis time.

JSON PROTOCOL SERVER
  saf run program.ll --serve

  Start SAF as a JSON protocol server for IDE integration.
  Accepts queries over stdin/stdout.

See also: saf help run, saf help checkers, saf help pta"
    );
}
