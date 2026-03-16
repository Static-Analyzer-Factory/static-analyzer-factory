# CruxBC Memory Optimization — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Fix OOM kills on large CruxBC programs (tmux, unrar) by reducing peak memory via intermediate drops and process-per-program isolation.

**Architecture:** Two complementary changes: (1) Insert explicit `drop()` calls and scoped blocks in `run_program_inner` to release intermediate data structures before building the next pipeline stage, (2) Add a hidden `CruxbcSingle` subcommand that analyzes one program and prints JSON; the main `cruxbc` command spawns a child process per program so the OS reclaims all memory between analyses.

**Tech Stack:** Rust, serde (Serialize + Deserialize), `std::process::Command`, clap hidden subcommands

**Root cause:** SAF runs all CruxBC programs in one process. For `extra` category programs (tmux=176K lines), the full pipeline (CG refinement + Andersen + MSSA + SVFG + FS-PTA) keeps all intermediate data structures alive simultaneously. Combined with allocator fragmentation from prior programs, this exceeds Docker's memory limit (exit 137). SVF avoids this by running each program in a fresh `docker run --rm` container.

---

### Task 1: Add `Deserialize` to CruxBC result types

`ProgramResult`, `IrStats`, `AnalysisStats` currently derive only `Serialize`. The parent process needs to deserialize child stdout, so all three need `Deserialize`.

**Files:**
- Modify: `crates/saf-bench/src/cruxbc.rs` (lines 16, 123, 142, 153)

**Step 1: Add `Deserialize` import and derives**

In `cruxbc.rs`, change the serde import:
```rust
// line 16: change
use serde::Serialize;
// to
use serde::{Deserialize, Serialize};
```

Add `Deserialize` to all three result structs:
```rust
// line 123: change
#[derive(Debug, Serialize)]
pub struct ProgramResult {
// to
#[derive(Debug, Serialize, Deserialize)]
pub struct ProgramResult {

// line 142: change
#[derive(Debug, Default, Serialize)]
pub struct IrStats {
// to
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct IrStats {

// line 153: change
#[derive(Debug, Default, Serialize)]
pub struct AnalysisStats {
// to
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AnalysisStats {
```

**Step 2: Verify**

Run: `make fmt && make lint`
Expected: PASS (no functional changes)

**Step 3: Commit**

```
feat(bench): add Deserialize to CruxBC result types

Needed for process-per-program isolation where parent deserializes
child process JSON output.
```

---

### Task 2: Drop intermediate data structures in `run_program_inner`

The full pipeline block (lines 436-459) keeps `refinement_result`, `defuse`, `cfgs`, `svfg`, and `_program_points` alive simultaneously with `mssa`, `fs_svfg`, and `_fs_result`. For tmux this is ~8-12GB peak. We can drop intermediates as soon as they're consumed.

**Files:**
- Modify: `crates/saf-bench/src/cruxbc.rs` (lines 378-459 in `run_program_inner`)

**Step 1: Drop `refinement_result` after extracting call graph**

After line 433 (`indirect_calls_resolved`), the `refinement_result` is no longer needed — we've extracted `constraint_counts`, `post_hvn_constraint_counts`, `pta_solve_secs`, and `resolved_sites`. But the call graph is still needed for MSSA/SVFG. Extract it before dropping:

Replace lines 435-459 (the full pipeline block) with:

```rust
    // --- Phases C + D: MSSA+SVFG and FS-PTA (small/extra only) ---
    if is_full_pipeline(&prog.category) {
        // Extract call graph, drop the rest of refinement_result
        let callgraph = refinement_result.call_graph;
        drop(refinement_result);

        // Phase C: MSSA + SVFG
        let t = Instant::now();
        let defuse = DefUseGraph::build(module);
        let cfgs: BTreeMap<FunctionId, Cfg> = module
            .functions
            .iter()
            .filter(|f| !f.is_declaration)
            .map(|f| (f.id, Cfg::build(f)))
            .collect();
        let mut mssa = MemorySsa::build(module, &cfgs, pta_result.clone(), &callgraph);

        // Drop cfgs — no longer needed after MSSA construction
        drop(cfgs);

        let (svfg, _program_points) =
            SvfgBuilder::new(module, &defuse, &callgraph, &pta_result, &mut mssa).build();
        phases.insert("mssa_svfg".to_string(), t.elapsed().as_secs_f64());

        // Drop defuse — no longer needed after SVFG construction
        drop(defuse);

        // Phase D: Flow-Sensitive PTA
        let t = Instant::now();
        let fs_svfg = FsSvfgBuilder::new(module, &svfg, &pta_result, &mut mssa, &callgraph).build();

        // Drop svfg — fs_svfg is self-contained
        drop(svfg);

        let fs_config = FsPtaConfig::default();
        let _fs_result =
            solve_flow_sensitive(module, &fs_svfg, &pta_result, &callgraph, &fs_config);
        phases.insert("fs_pta".to_string(), t.elapsed().as_secs_f64());
    } else {
        drop(refinement_result);
    }
```

Note: the `else` branch explicitly drops `refinement_result` for the non-full-pipeline case to free memory sooner.

**Step 2: Verify**

Run: `make fmt && make lint`
Expected: PASS

Run: `make test-cruxbc FILTER=small`
Expected: All small programs complete successfully with same results

**Step 3: Commit**

```
perf(bench): drop intermediate data structures in CruxBC pipeline

Drop refinement_result, cfgs, defuse, and svfg as soon as they're
consumed to reduce peak memory. For tmux (176K lines), the full
pipeline previously kept all structures alive simultaneously.
```

---

### Task 3: Add `CruxbcSingle` hidden subcommand

Add a hidden subcommand that analyzes a single program and writes `ProgramResult` JSON to stdout. This is the child process entry point for process isolation.

**Files:**
- Modify: `crates/saf-bench/src/main.rs` (add `CruxbcSingle` variant to `Commands` enum + handler)
- Modify: `crates/saf-bench/src/cruxbc.rs` (make `run_single_program` public)

**Step 1: Make `run_single_program` and `CruxBcProgram` public**

In `cruxbc.rs`, change `CruxBcProgram` to public with public fields:

```rust
// line 205: change
struct CruxBcProgram {
    path: PathBuf,
    category: String,
    name: String,
}
// to
/// A discovered program to benchmark.
pub struct CruxBcProgram {
    /// Path to the .ll/.bc file.
    pub path: PathBuf,
    /// Category (directory name, e.g., "small", "big", "extra").
    pub category: String,
    /// Program name (filename without extension).
    pub name: String,
}
```

Change `run_single_program` to public:

```rust
// line 280: change
fn run_single_program(prog: &CruxBcProgram, solver: PtaSolver) -> ProgramResult {
// to
/// Run all analysis phases on a single program, catching panics.
pub fn run_single_program(prog: &CruxBcProgram, solver: PtaSolver) -> ProgramResult {
```

**Step 2: Add `CruxbcSingle` subcommand to `main.rs`**

In the `Commands` enum (after `Cruxbc` variant around line 188), add:

```rust
    /// Run a single CruxBC program (internal — used for process isolation)
    #[command(hide = true)]
    CruxbcSingle {
        /// Path to the .ll/.bc file
        #[arg(long)]
        path: PathBuf,

        /// Category (small/big/extra)
        #[arg(long)]
        category: String,

        /// PTA solver: "worklist" or "datalog"
        #[arg(long, default_value = "worklist")]
        solver: String,
    },
```

**Step 3: Add handler in `main()`**

In the `match cli.command` block (after the `Commands::Cruxbc` arm), add:

```rust
        Commands::CruxbcSingle {
            path,
            category,
            solver,
        } => {
            let pta_solver = match solver.as_str() {
                "worklist" | "legacy" => PtaSolver::Worklist,
                _ => PtaSolver::Datalog,
            };
            let name = path
                .file_stem()
                .map_or_else(|| "unknown".to_string(), |s| s.to_string_lossy().to_string());
            let prog = saf_bench::cruxbc::CruxBcProgram {
                path,
                category,
                name,
            };
            let result = saf_bench::cruxbc::run_single_program(&prog, pta_solver);
            println!("{}", serde_json::to_string(&result)?);
        }
```

**Step 4: Verify**

Run: `make fmt && make lint`
Expected: PASS

**Step 5: Commit**

```
feat(bench): add hidden cruxbc-single subcommand for process isolation

Analyzes a single program and prints ProgramResult JSON to stdout.
Used internally by the cruxbc runner to isolate each analysis in its
own process, ensuring the OS reclaims all memory between programs.
```

---

### Task 4: Implement process-per-program orchestration in `CruxBcRunner`

Modify `CruxBcRunner::run()` to spawn a child process per program instead of running them in-process. Each child re-invokes the same binary with `cruxbc-single`.

**Files:**
- Modify: `crates/saf-bench/src/cruxbc.rs` (`CruxBcRunner::run()` method, lines 72-103)

**Step 1: Rewrite `CruxBcRunner::run()`**

Replace the existing `run()` method with:

```rust
    /// Run the benchmark suite and return a summary.
    ///
    /// Each program is analyzed in a separate child process to ensure the OS
    /// reclaims all memory between analyses. The child invokes `saf-bench
    /// cruxbc-single` and writes `ProgramResult` JSON to stdout.
    ///
    /// # Errors
    ///
    /// Returns an error if the current executable path cannot be determined.
    pub fn run(&self) -> Result<CruxBcSummary> {
        let overall_start = Instant::now();

        let programs = discover_programs(&self.config.compiled_dir, self.config.filter.as_deref());
        eprintln!(
            "Discovered {} programs in {}",
            programs.len(),
            self.config.compiled_dir.display()
        );

        let exe = std::env::current_exe().context("failed to determine current executable")?;
        let solver_str = format!("{:?}", self.config.solver).to_lowercase();

        let results: Vec<ProgramResult> = programs
            .iter()
            .map(|prog| run_isolated(&exe, prog, &solver_str))
            .collect();

        Ok(CruxBcSummary {
            suite: "cruxbc".to_string(),
            solver: format!("{:?}", self.config.solver),
            programs: results,
            total_secs: overall_start.elapsed().as_secs_f64(),
        })
    }
```

**Step 2: Add `run_isolated` helper function**

Add this function after `run_single_program` (around line 324):

```rust
/// Run a single program in an isolated child process.
///
/// Spawns `saf-bench cruxbc-single` as a subprocess. On success, parses the
/// `ProgramResult` JSON from stdout. On failure (crash, OOM, timeout),
/// returns a `ProgramResult` with the error message.
fn run_isolated(exe: &Path, prog: &CruxBcProgram, solver: &str) -> ProgramResult {
    let label = format!("{}/{}", prog.category, prog.name);
    eprintln!("  Analyzing {label} (isolated, {solver}) ...");

    let start = Instant::now();

    let output = std::process::Command::new(exe)
        .arg("cruxbc-single")
        .arg("--path")
        .arg(&prog.path)
        .arg("--category")
        .arg(&prog.category)
        .arg("--solver")
        .arg(solver)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::inherit())
        .output();

    let elapsed = start.elapsed().as_secs_f64();

    match output {
        Ok(out) if out.status.success() => {
            match serde_json::from_slice::<ProgramResult>(&out.stdout) {
                Ok(pr) => {
                    eprintln!("  {label}: {:.2}s", pr.total_secs);
                    pr
                }
                Err(e) => {
                    let msg = format!("failed to parse child JSON: {e}");
                    eprintln!("  {label}: ERROR {msg}");
                    error_result(&prog.name, &prog.category, msg)
                }
            }
        }
        Ok(out) => {
            let code = out.status.code().unwrap_or(-1);
            let msg = if code == 137 {
                format!("killed by OOM (exit {code}, {elapsed:.1}s)")
            } else {
                let stderr_tail = String::from_utf8_lossy(&out.stderr);
                let tail: String = stderr_tail.chars().rev().take(500).collect::<String>().chars().rev().collect();
                format!("child exited with code {code}: {tail}")
            };
            eprintln!("  {label}: ERROR {msg}");
            error_result(&prog.name, &prog.category, msg)
        }
        Err(e) => {
            let msg = format!("failed to spawn child: {e}");
            eprintln!("  {label}: ERROR {msg}");
            error_result(&prog.name, &prog.category, msg)
        }
    }
}

/// Create a `ProgramResult` representing an error.
fn error_result(name: &str, category: &str, error: String) -> ProgramResult {
    ProgramResult {
        name: name.to_string(),
        category: category.to_string(),
        ir_stats: IrStats::default(),
        phases: BTreeMap::new(),
        total_secs: 0.0,
        error: Some(error),
        stats: AnalysisStats::default(),
    }
}
```

**Step 3: Remove `rayon` from `run()` and the unused import in `cruxbc.rs`**

The `rayon` import and `jobs` field were only used in the old in-process parallel mode. Process isolation makes parallel execution via rayon unnecessary (each child is already a separate process; the parent is sequential and cheap).

However, keep the `jobs` field in `CruxBcConfig` for now — we can remove it in a follow-up if desired. Just remove the rayon usage from `run()` (which we already did above by removing the `if self.config.jobs > 1` branch).

**Step 4: Verify**

Run: `make fmt && make lint`
Expected: PASS (may get unused `jobs` warning — add `#[allow(dead_code)]` on the field or prefix with `_`)

Run: `make test-cruxbc FILTER=small`
Expected: All small programs complete. Output format unchanged.

**Step 5: Commit**

```
feat(bench): run each CruxBC program in isolated child process

Spawn `saf-bench cruxbc-single` per program so the OS reclaims all
memory between analyses. Fixes OOM kills on large extra-category
programs (tmux, unrar) where the full pipeline (MSSA+SVFG+FS-PTA)
exceeded Docker memory limits.

If a child is OOM-killed (exit 137), the error is reported in the
results rather than crashing the entire benchmark suite.
```

---

### Task 5: Remove unused rayon import from `cruxbc.rs`

After Task 4, the `rayon` import and parallel branch are dead code in `cruxbc.rs`.

**Files:**
- Modify: `crates/saf-bench/src/cruxbc.rs`

**Step 1: Remove rayon usage**

If rayon is still imported in cruxbc.rs, remove it. Also check if `jobs` field triggers a dead code warning; if so, either:
- Remove the `jobs` field from `CruxBcConfig` and update `main.rs` accordingly, OR
- Add `#[allow(dead_code)]` to the field

The `rayon` crate dependency in `Cargo.toml` should stay because `main.rs` uses it for PTABen parallel runs.

**Step 2: Verify**

Run: `make fmt && make lint`
Expected: PASS, no warnings

**Step 3: Commit**

```
refactor(bench): remove unused rayon import from cruxbc module
```

---

### Task 6: End-to-end verification

**Step 1: Run small programs**

Run: `make test-cruxbc FILTER=small`
Expected: All small programs pass. Verify JSON output matches previous format.

**Step 2: Run extra programs (including tmux)**

Run: `make test-cruxbc FILTER=extra`
Expected: tmux either completes or reports OOM gracefully in results (not a full crash). bzip2recover and curl should complete.

**Step 3: Run full suite**

Run: `make test-cruxbc`
Expected: All programs produce results. Any OOM errors are captured per-program, not suite-wide crashes.

**Step 4: Verify JSON output structure**

Read `tests/benchmarks/cruxbc/saf-results.json` and verify:
- All programs have entries
- Successful programs have phases and stats
- Failed programs have error messages

**Step 5: Commit (if any fixups needed)**

**Step 6: Update PROGRESS.md**

Add plan 168 to the Plans Index and update the Session Log.
