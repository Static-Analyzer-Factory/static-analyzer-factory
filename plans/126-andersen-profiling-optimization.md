# Plan 126: Profile-Guided Andersen PTA Optimization

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Instrument the Andersen PTA solver to identify where the 44s bash benchmark time is actually spent, then apply targeted optimizations based on evidence.

**Architecture:** Add a feature-gated `SolverStats` struct to `GenericSolver` that accumulates timing and counters during solving. Wire it into the CruxBC benchmark runner. Profile bash, analyze results, then implement the top 1-2 optimizations the data reveals. Previous Plans 124-125 failed because they optimized blind — this plan measures first.

**Tech Stack:** Rust, `std::time::Instant` for timing, Cargo feature flag `solver-stats`.

**Key risk:** Instrumentation overhead could distort results. Mitigation: use `Instant::now()` only around coarse-grained sections (per-handler, not per-BTreeSet-op), and sample pts-set statistics rather than measuring every operation.

---

## Results

### Baseline Profile (bash, before optimization)

```
Phase breakdown (cumulative wall-clock, ~22.0s total):
  handle_load:   16.26s  (73.8%)  — 2,897,402 locs iterated (2.9M BTreeSet clones)
  diff_compute:   2.06s  ( 9.4%)
  handle_gep:     1.24s  ( 5.6%)
  handle_store:   1.17s  ( 5.3%)
  handle_copy:    0.36s  ( 1.6%)
  scc_detect:     0.11s  ( 0.5%)
  lcd_check:      0.04s  ( 0.2%)
  worklist_ops:   0.02s  ( 0.1%)

bash Ander: 46.40s, Total: 63.94s
```

### Post-Optimization Profile (load handler accumulation)

```
Phase breakdown (cumulative wall-clock, 15.44s total):
  handle_load:   10.69s  (69.3%)  — 2,897,402 locs iterated (no clones)
  diff_compute:   1.94s  (12.5%)
  handle_gep:     1.19s  ( 7.7%)
  handle_store:   1.12s  ( 7.3%)
  handle_copy:    0.33s  ( 2.2%)
  scc_detect:     0.10s  ( 0.7%)
  lcd_check:      0.04s  ( 0.2%)
  worklist_ops:   0.02s  ( 0.1%)

bash Ander: 43.02s (-7.3%), Total: 60.87s (-4.8%)
```

### Key Metrics
- **handle_load improvement:** 16.26s → 10.69s (**-34%**, -5.57s)
- **Solver total improvement:** ~22.0s → 15.44s (**-30%**, -6.6s)
- **bash Ander improvement:** 46.40s → 43.02s (**-7.3%**, -3.38s)
- **PTABen regression:** None (69 Unsound unchanged)
- **Test suite:** 1417 passed, 5 skipped

### Analysis
The load handler is still the dominant bottleneck (69.3%). The optimization eliminated 2.9M BTreeSet clones per solve by accumulating loc_pts into a single set before union. The Ander improvement (3.38s) is less than the solver improvement (6.6s) because the profiled run is within CG refinement, and additional unprofiled solver time exists. Remaining optimization candidates: diff_compute (12.5%), handle_gep (7.7%), handle_store (7.3%).

---

## Baseline (record before implementing)

Run CruxBC benchmarks on bash to establish timing baseline:

```bash
docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- cruxbc --compiled-dir tests/benchmarks/cruxbc/.compiled --filter "big/bash" -o /workspace/tests/benchmarks/cruxbc/baseline-126.json'
```

Expected: bash Andersen ~44-48s.

---

### Task 1: Add `solver-stats` Feature and `SolverStats` Struct

**Files:**
- Modify: `crates/saf-analysis/Cargo.toml` (add feature)
- Create: `crates/saf-analysis/src/pta/solver_stats.rs` (~120 lines)
- Modify: `crates/saf-analysis/src/pta/mod.rs` (add module)

**Step 1: Add the `solver-stats` feature flag**

In `crates/saf-analysis/Cargo.toml`, add to the `[features]` section:

```toml
solver-stats = []
```

If there is no `[features]` section, create one. This is a compile-time feature with no additional dependencies.

**Step 2: Create the stats module**

Create `crates/saf-analysis/src/pta/solver_stats.rs`:

```rust
//! Solver profiling statistics (feature-gated behind `solver-stats`).
//!
//! Accumulates timing and counters during Andersen PTA solving to identify
//! performance bottlenecks. Zero cost when the feature is disabled.

use std::time::{Duration, Instant};

/// Accumulated profiling statistics for a single solver run.
#[derive(Debug, Default)]
pub struct SolverStats {
    // === Phase timing (cumulative wall-clock) ===
    /// Time in `handle_copy_constraints`.
    pub time_copy: Duration,
    /// Time in `handle_load_constraints`.
    pub time_load: Duration,
    /// Time in `handle_store_constraints`.
    pub time_store: Duration,
    /// Time in `handle_gep_constraints`.
    pub time_gep: Duration,
    /// Time computing diffs in `process_value` (clone + difference + is_empty).
    pub time_diff: Duration,
    /// Time in `detect_and_collapse_cycles` (periodic Tarjan).
    pub time_scc: Duration,
    /// Time in `check_pending_cycles` (LCD).
    pub time_lcd: Duration,
    /// Time in worklist insert/pop operations.
    pub time_worklist: Duration,

    // === Operation counts ===
    /// Total values popped from the worklist.
    pub value_pops: u64,
    /// Total locations popped from the loc worklist.
    pub loc_pops: u64,
    /// Total `find_rep` calls.
    pub find_rep_calls: u64,
    /// Maximum chain length seen in any `find_rep` call.
    pub find_rep_max_chain: u32,
    /// Total cumulative hops across all `find_rep` calls.
    pub find_rep_total_hops: u64,
    /// Total `union` calls (on PtsSet).
    pub union_calls: u64,
    /// Union calls that actually grew the target set.
    pub union_changed: u64,
    /// Times `process_value` returned early due to empty diff.
    pub empty_diff_skips: u64,
    /// Times `process_value` was called (before early return).
    pub process_value_calls: u64,
    /// Number of SCC detection invocations.
    pub scc_invocations: u32,
    /// Number of nodes merged by SCC detection.
    pub scc_merges: u32,
    /// Number of LCD check invocations.
    pub lcd_invocations: u32,
    /// Number of nodes merged by LCD.
    pub lcd_merges: u32,
    /// Total locations iterated in load handler.
    pub load_locs_iterated: u64,
    /// Total locations iterated in store handler.
    pub store_locs_iterated: u64,
    /// Total locations iterated in GEP handler.
    pub gep_locs_iterated: u64,
    /// Total constraint indices looked up in copy handler.
    pub copy_indices_processed: u64,
}

impl SolverStats {
    /// Start a timing section. Returns an `Instant` to pass to `end_section`.
    #[inline]
    pub fn start_section() -> Instant {
        Instant::now()
    }

    /// End a timing section and accumulate into the given duration field.
    #[inline]
    pub fn end_section(start: Instant, accum: &mut Duration) {
        *accum += start.elapsed();
    }

    /// Print a human-readable profile summary to stderr.
    pub fn print_summary(&self, label: &str, constraint_counts: (usize, usize, usize, usize)) {
        let (copy_c, load_c, store_c, gep_c) = constraint_counts;
        let total_time = self.time_copy
            + self.time_load
            + self.time_store
            + self.time_gep
            + self.time_diff
            + self.time_scc
            + self.time_lcd
            + self.time_worklist;
        let total_secs = total_time.as_secs_f64();

        let pct = |d: Duration| -> f64 {
            if total_secs > 0.0 {
                d.as_secs_f64() / total_secs * 100.0
            } else {
                0.0
            }
        };

        eprintln!();
        eprintln!("=== Andersen Solver Profile ({label}) ===");
        eprintln!(
            "Constraints: copy={copy_c} load={load_c} store={store_c} gep={gep_c} total={}",
            copy_c + load_c + store_c + gep_c
        );
        eprintln!(
            "Worklist: {} value pops, {} loc pops",
            self.value_pops, self.loc_pops
        );
        eprintln!(
            "process_value: {} calls, {} empty-diff skips ({:.0}%)",
            self.process_value_calls,
            self.empty_diff_skips,
            if self.process_value_calls > 0 {
                self.empty_diff_skips as f64 / self.process_value_calls as f64 * 100.0
            } else {
                0.0
            }
        );
        eprintln!();
        eprintln!("Phase breakdown (cumulative wall-clock, {total_secs:.2}s total):");
        eprintln!(
            "  handle_copy:   {:>6.2}s  ({:>4.1}%)  \u{2014} {} indices processed, {} union calls, {} changed",
            self.time_copy.as_secs_f64(),
            pct(self.time_copy),
            self.copy_indices_processed,
            self.union_calls,
            self.union_changed,
        );
        eprintln!(
            "  handle_load:   {:>6.2}s  ({:>4.1}%)  \u{2014} {} locs iterated",
            self.time_load.as_secs_f64(),
            pct(self.time_load),
            self.load_locs_iterated,
        );
        eprintln!(
            "  handle_store:  {:>6.2}s  ({:>4.1}%)  \u{2014} {} locs iterated",
            self.time_store.as_secs_f64(),
            pct(self.time_store),
            self.store_locs_iterated,
        );
        eprintln!(
            "  handle_gep:    {:>6.2}s  ({:>4.1}%)  \u{2014} {} locs iterated",
            self.time_gep.as_secs_f64(),
            pct(self.time_gep),
            self.gep_locs_iterated,
        );
        eprintln!(
            "  diff_compute:  {:>6.2}s  ({:>4.1}%)",
            self.time_diff.as_secs_f64(),
            pct(self.time_diff),
        );
        eprintln!(
            "  scc_detect:    {:>6.2}s  ({:>4.1}%)  \u{2014} {} invocations, {} merges",
            self.time_scc.as_secs_f64(),
            pct(self.time_scc),
            self.scc_invocations,
            self.scc_merges,
        );
        eprintln!(
            "  lcd_check:     {:>6.2}s  ({:>4.1}%)  \u{2014} {} invocations, {} merges",
            self.time_lcd.as_secs_f64(),
            pct(self.time_lcd),
            self.lcd_invocations,
            self.lcd_merges,
        );
        eprintln!(
            "  worklist_ops:  {:>6.2}s  ({:>4.1}%)",
            self.time_worklist.as_secs_f64(),
            pct(self.time_worklist),
        );
        eprintln!();
        eprintln!(
            "find_rep: {} calls, {} total hops, max chain={}, avg hops={:.2}",
            self.find_rep_calls,
            self.find_rep_total_hops,
            self.find_rep_max_chain,
            if self.find_rep_calls > 0 {
                self.find_rep_total_hops as f64 / self.find_rep_calls as f64
            } else {
                0.0
            }
        );
    }
}
```

**Step 3: Register the module**

In `crates/saf-analysis/src/pta/mod.rs`, add:

```rust
#[cfg(feature = "solver-stats")]
pub(crate) mod solver_stats;
#[cfg(feature = "solver-stats")]
pub use solver_stats::SolverStats;
```

**Step 4: Verify compilation**

```bash
docker compose run --rm dev sh -c 'cargo check -p saf-analysis && cargo check -p saf-analysis --features solver-stats'
```

Expected: both compile without errors.

**Step 5: Commit**

```bash
git add crates/saf-analysis/Cargo.toml crates/saf-analysis/src/pta/solver_stats.rs crates/saf-analysis/src/pta/mod.rs
git commit -m "feat(pta): add solver-stats feature with SolverStats profiling struct"
```

---

### Task 2: Instrument the Solver

**Files:**
- Modify: `crates/saf-analysis/src/pta/solver.rs`

This task adds conditional instrumentation to every hot-path method in `GenericSolver`. All instrumentation is gated behind `#[cfg(feature = "solver-stats")]`.

**Step 1: Add the stats field to `GenericSolver`**

Add to the `GenericSolver` struct definition (after the `topo_order` field):

```rust
    /// Profiling statistics (only present when `solver-stats` feature is enabled).
    #[cfg(feature = "solver-stats")]
    pub(crate) stats: SolverStats,
```

Update both constructors (`new` and `new_with_template`) to initialize it:

```rust
    #[cfg(feature = "solver-stats")]
    stats: SolverStats::default(),
```

Add the import at the top of solver.rs (conditionally):

```rust
#[cfg(feature = "solver-stats")]
use super::solver_stats::SolverStats;
```

**Step 2: Instrument `find_rep`**

Replace the `find_rep` method body with:

```rust
fn find_rep(&self, mut v: ValueId) -> ValueId {
    #[cfg(feature = "solver-stats")]
    let mut hops: u32 = 0;

    while let Some(&r) = self.rep.get(&v) {
        if r == v {
            break;
        }
        v = r;
        #[cfg(feature = "solver-stats")]
        {
            hops += 1;
        }
    }

    #[cfg(feature = "solver-stats")]
    {
        // SAFETY: We use unsafe to mutate stats through &self. This is safe
        // because find_rep is only called from single-threaded solver methods,
        // and stats are write-only counters that don't affect solver correctness.
        let stats = unsafe {
            &mut *(&self.stats as *const SolverStats as *mut SolverStats)
        };
        stats.find_rep_calls += 1;
        stats.find_rep_total_hops += u64::from(hops);
        if hops > stats.find_rep_max_chain {
            stats.find_rep_max_chain = hops;
        }
    }

    v
}
```

Note: We need interior mutability because `find_rep` takes `&self`. Using raw pointer cast is safe here because the solver is single-threaded and stats are write-only counters. An alternative would be `Cell`/`UnsafeCell` but that changes the struct layout for all configurations.

**Step 3: Instrument `process_value`**

In `process_value`, add instrumentation around each section. Replace the method body:

```rust
fn process_value(&mut self, v: ValueId) {
    let v = self.find_rep(v);

    #[cfg(feature = "solver-stats")]
    {
        self.stats.process_value_calls += 1;
    }

    let current = match self.pts.get(&v) {
        Some(s) => s.clone(),
        None => return,
    };

    // Compute diff: elements in current but not in prev
    #[cfg(feature = "solver-stats")]
    let diff_start = SolverStats::start_section();

    let diff = if let Some(prev) = self.prev_pts.get(&v) {
        let mut d = current.clone();
        d.difference(prev);
        d
    } else {
        current.clone()
    };

    #[cfg(feature = "solver-stats")]
    SolverStats::end_section(diff_start, &mut self.stats.time_diff);

    if diff.is_empty() {
        #[cfg(feature = "solver-stats")]
        {
            self.stats.empty_diff_skips += 1;
        }
        return; // Nothing new to propagate
    }

    // Update prev snapshot
    self.prev_pts.insert(v, current);

    // --- Copy ---
    #[cfg(feature = "solver-stats")]
    let t = SolverStats::start_section();
    self.handle_copy_constraints(v, &diff);
    #[cfg(feature = "solver-stats")]
    SolverStats::end_section(t, &mut self.stats.time_copy);

    // --- Load ---
    #[cfg(feature = "solver-stats")]
    let t = SolverStats::start_section();
    self.handle_load_constraints(v, &diff);
    #[cfg(feature = "solver-stats")]
    SolverStats::end_section(t, &mut self.stats.time_load);

    // --- Store ---
    #[cfg(feature = "solver-stats")]
    let t = SolverStats::start_section();
    self.handle_store_constraints(v, &diff);
    #[cfg(feature = "solver-stats")]
    SolverStats::end_section(t, &mut self.stats.time_store);

    // --- GEP ---
    #[cfg(feature = "solver-stats")]
    let t = SolverStats::start_section();
    self.handle_gep_constraints(v, &diff);
    #[cfg(feature = "solver-stats")]
    SolverStats::end_section(t, &mut self.stats.time_gep);
}
```

**Step 4: Instrument `drain_worklist`**

Add worklist timing and pop counters. In the main loop, wrap the `pop_first` and `worklist_insert` calls. In the value processing section:

```rust
pub(crate) fn drain_worklist(&mut self, max_iterations: usize) {
    let mut iterations = 0;
    let mut value_pops = 0u64;

    while iterations < max_iterations {
        // Process all values at the current minimum rank (wave front)
        if let Some(&(current_rank, _)) = self.worklist.first() {
            let mut wave: Vec<ValueId> = Vec::new();

            #[cfg(feature = "solver-stats")]
            let wl_start = SolverStats::start_section();

            while let Some(&(rank, _)) = self.worklist.first() {
                if rank != current_rank {
                    break;
                }
                let (_, v) = self.worklist.pop_first().expect("just peeked");
                wave.push(v);
            }

            #[cfg(feature = "solver-stats")]
            SolverStats::end_section(wl_start, &mut self.stats.time_worklist);

            for v in wave {
                iterations += 1;
                if iterations >= max_iterations {
                    return;
                }
                value_pops += 1;

                #[cfg(feature = "solver-stats")]
                {
                    self.stats.value_pops += 1;
                }

                if value_pops % 50_000 == 0 {
                    #[cfg(feature = "solver-stats")]
                    let scc_start = SolverStats::start_section();

                    self.detect_and_collapse_cycles();

                    #[cfg(feature = "solver-stats")]
                    SolverStats::end_section(scc_start, &mut self.stats.time_scc);
                }
                self.process_value(v);
            }

            #[cfg(feature = "solver-stats")]
            let lcd_start = SolverStats::start_section();

            self.check_pending_cycles();

            #[cfg(feature = "solver-stats")]
            SolverStats::end_section(lcd_start, &mut self.stats.time_lcd);

            continue;
        }

        // Process location worklist (between waves)
        if let Some(loc) = self.loc_worklist.pop_first() {
            iterations += 1;

            #[cfg(feature = "solver-stats")]
            {
                self.stats.loc_pops += 1;
            }

            self.process_location(loc);
            continue;
        }

        // Both worklists empty -- fixed point reached
        break;
    }
}
```

**Step 5: Instrument `handle_copy_constraints` for union counting**

Add per-union tracking:

```rust
fn handle_copy_constraints(&mut self, v: ValueId, v_pts: &P) {
    let indices: Vec<usize> = self.index.copies_by_src(v).to_vec();

    #[cfg(feature = "solver-stats")]
    {
        self.stats.copy_indices_processed += indices.len() as u64;
    }

    for i in indices {
        let dst = self.indexed.copy[i].dst;

        #[cfg(feature = "solver-stats")]
        {
            self.stats.union_calls += 1;
        }

        if self.union_into_value(dst, v_pts) {
            #[cfg(feature = "solver-stats")]
            {
                self.stats.union_changed += 1;
            }

            self.worklist_insert(dst);
            self.pending_cycle_pairs.push((v, dst));
        }
    }
}
```

**Step 6: Instrument `handle_load_constraints`, `handle_store_constraints`, `handle_gep_constraints`**

Add loc-iteration counters to each. For load:

```rust
fn handle_load_constraints(&mut self, v: ValueId, v_pts: &P) {
    let indices: Vec<usize> = self.index.loads_by_src_ptr(v).to_vec();
    for i in &indices {
        let dst = self.indexed.load[*i].dst;
        for loc in v_pts.iter() {
            #[cfg(feature = "solver-stats")]
            {
                self.stats.load_locs_iterated += 1;
            }
            // ... rest unchanged
```

For store, add `self.stats.store_locs_iterated += 1;` inside each `for loc in ...` loop (both the dst_ptr loop and the src loop).

For GEP, add `self.stats.gep_locs_iterated += 1;` inside the `for loc in v_pts.iter()` loop.

**Step 7: Instrument SCC/LCD merge counters**

In `detect_and_collapse_cycles`, after the merge loop:

```rust
// Collapse SCCs with >1 node
for scc in &sccs {
    if scc.len() <= 1 {
        continue;
    }
    #[cfg(feature = "solver-stats")]
    {
        self.stats.scc_invocations += 1;
    }
    let rep = *scc.iter().min().expect("SCC is non-empty");
    for &node in scc {
        if node != rep {
            self.merge_nodes(rep, node);
            #[cfg(feature = "solver-stats")]
            {
                self.stats.scc_merges += 1;
            }
        }
    }
}
```

In `check_pending_cycles`, increment `lcd_invocations` at the start (if pairs is non-empty), and `lcd_merges` each time `merge_nodes` is called.

**Step 8: Add `print_stats` method to `GenericSolver`**

```rust
#[cfg(feature = "solver-stats")]
impl<'a, P: PtsSet> GenericSolver<'a, P> {
    /// Print profiling statistics to stderr.
    pub(crate) fn print_stats(&self, label: &str) {
        let constraint_counts = (
            self.indexed.copy.len(),
            self.indexed.load.len(),
            self.indexed.store.len(),
            self.indexed.gep.len(),
        );
        self.stats.print_summary(label, constraint_counts);
    }
}
```

**Step 9: Verify compilation (both with and without feature)**

```bash
docker compose run --rm dev sh -c 'cargo check -p saf-analysis && cargo check -p saf-analysis --features solver-stats'
```

Expected: both compile without errors.

**Step 10: Run tests without the feature (no regression)**

```bash
docker compose run --rm dev sh -c 'cargo nextest run -p saf-analysis 2>&1 | tail -5'
```

Expected: all tests pass (stats have no effect when feature is disabled).

**Step 11: Commit**

```bash
git add crates/saf-analysis/src/pta/solver.rs
git commit -m "feat(pta): instrument solver hot paths for profiling (solver-stats feature)"
```

---

### Task 3: Wire Profiling into CG Refinement and CruxBC Runner

**Files:**
- Modify: `crates/saf-analysis/src/cg_refinement.rs`
- Modify: `crates/saf-bench/Cargo.toml`
- Modify: `crates/saf-bench/src/cruxbc.rs`

**Step 1: Add `print_stats` call in `cg_refinement::refine`**

In `crates/saf-analysis/src/cg_refinement.rs`, after the refinement loop completes (after the `for _wave in ...` loop, around line 179), add:

```rust
    // Print solver profile if instrumented
    #[cfg(feature = "solver-stats")]
    solver.print_stats("cg-refinement");
```

**Step 2: Forward the feature in saf-bench**

In `crates/saf-bench/Cargo.toml`, add:

```toml
[features]
solver-stats = ["saf-analysis/solver-stats"]
```

**Step 3: Verify compilation with the feature**

```bash
docker compose run --rm dev sh -c 'cargo check -p saf-bench --features solver-stats'
```

Expected: compiles without errors.

**Step 4: Run the profiled benchmark on bash**

```bash
docker compose run --rm dev sh -c 'cargo run --release -p saf-bench --features solver-stats -- cruxbc --compiled-dir tests/benchmarks/cruxbc/.compiled --filter "big/bash" -o /workspace/tests/benchmarks/cruxbc/profile-126.json 2>&1 | tee /tmp/profile-126-stderr.txt'
```

This outputs the JSON results to a file AND captures the stderr profile output (which contains the `SolverStats` summary) to `/tmp/profile-126-stderr.txt`.

**Step 5: Read and analyze the profile output**

Read `/tmp/profile-126-stderr.txt` and look for the `=== Andersen Solver Profile ===` section. Record:
- Which handler takes the most time
- The empty-diff skip ratio
- The find_rep call count and avg hops
- The union changed ratio
- The pts-size distribution

**Step 6: Commit**

```bash
git add crates/saf-analysis/src/cg_refinement.rs crates/saf-bench/Cargo.toml
git commit -m "feat(bench): wire solver-stats profiling into CG refinement and CruxBC runner"
```

---

### Task 4: Analyze Profile Results and Identify Optimization Targets

This is a research/analysis task, not a coding task. **Do not write code.**

**Step 1: Examine the profile data from Task 3 Step 5**

Read `/tmp/profile-126-stderr.txt` and extract the key metrics.

**Step 2: Identify the top 2-3 bottlenecks**

Use this decision matrix:

| If the profile shows... | Likely optimization |
|---|---|
| `handle_copy` > 40% of total | BTreeSet union is hot. Implement merge-based union or sorted-Vec PtsSet. |
| `time_diff` > 15% | Clone + difference overhead. Use incremental diff tracking or COW sets. |
| `handle_load` > 30% | Load-through-pointer propagation doing too much work. Batch or cache. |
| `handle_store` > 20% | Store propagation hot. Check if dst_ptr pts sets are large. |
| `handle_gep` > 15% | GEP/field deriving is expensive. Cache resolved paths. |
| `empty_diff_skips` > 60% | Worklist is over-scheduling. Better change-tracking could skip earlier. |
| `find_rep avg hops` > 2.0 | Path compression would help. Use `&mut self` approach (not Cell). |
| `union_changed / union_calls` < 20% | Most unions are redundant. Better worklist ordering or pre-filtering. |
| `scc_detect` > 5% | Tarjan interval too aggressive or SCC detection overhead high. |
| `worklist_ops` > 10% | BTreeSet priority queue is expensive. Switch to BinaryHeap. |

**Step 3: Document findings**

Record the profile results and chosen optimization targets in this plan document's Results section (to be added after profiling). Update `plans/PROGRESS.md` with findings.

**Step 4: No commit needed (analysis only)**

---

### Task 5: Implement Optimization #1 (Based on Profile Data)

**This task's content depends on Task 4's findings.** The plan provides the three most likely optimizations based on prior knowledge of the solver architecture. Implement whichever the profile data points to.

#### Option A: Merge-Based BTreeSet Union (if `handle_copy` dominates)

**Files:**
- Modify: `crates/saf-analysis/src/pta/ptsset/btree.rs`

The current `BTreePtsSet::union` uses `extend()` which calls `insert()` per element — O(n log(n+m)). Since both sets are sorted (BTreeSet), a merge-based approach is O(n+m):

```rust
fn union(&mut self, other: &Self) -> bool {
    if other.inner.is_empty() {
        return false;
    }
    if self.inner.is_empty() {
        self.inner = other.inner.clone();
        return true;
    }
    let old_len = self.inner.len();
    // BTreeSet::extend is already merge-optimized in Rust std since 1.59
    // but we can check if a simple append + sort approach is faster for small sets
    self.inner.extend(other.inner.iter().copied());
    self.inner.len() > old_len
}
```

Actually, `BTreeSet::extend` on sorted input IS already O(n+m) in modern Rust. The real win may be in avoiding the `extend` entirely when `other` is a subset — add an early-exit check:

```rust
fn union(&mut self, other: &Self) -> bool {
    if other.inner.is_empty() {
        return false;
    }
    // Fast path: if other is a single element (very common for sparse sets)
    if other.inner.len() == 1 {
        let loc = *other.inner.iter().next().expect("len==1");
        return self.inner.insert(loc);
    }
    let old_len = self.inner.len();
    self.inner.extend(other.inner.iter().copied());
    self.inner.len() > old_len
}
```

#### Option B: FxHashMap for Solver Maps (if IndexMap lookups are hot)

**Files:**
- Modify: `crates/saf-analysis/Cargo.toml` (add `rustc-hash` dependency)
- Modify: `crates/saf-analysis/src/pta/solver.rs` (replace IndexMap with FxIndexMap)

Add `rustc-hash = "2"` to dependencies. Then in solver.rs, replace:
- `IndexMap<ValueId, P>` with `IndexMap<ValueId, P, FxBuildHasher>`
- Same for `loc_pts`, `prev_pts`, `rep`, `topo_order`, `load_loc_index`

This changes the hash function from SipHash (13ns for u128) to FxHash (~2ns for u128).

Note: `indexmap` supports custom hashers via its `BuildHasher` generic parameter. Import via `use rustc_hash::FxBuildHasher;`.

#### Option C: Avoid Clone in Diff Computation (if `time_diff` dominates)

**Files:**
- Modify: `crates/saf-analysis/src/pta/solver.rs`

The current diff computation in `process_value` does:
1. `current.clone()` — full BTreeSet clone
2. `d.difference(prev)` — in-place retain
3. If empty, discard both clones

Alternative: check subset first (cheap for small sets), only clone if needed:

```rust
// Instead of clone-then-diff, check if current == prev first
let has_new = if let Some(prev) = self.prev_pts.get(&v) {
    // For small sets, direct comparison is faster than clone+diff
    !current.is_subset(prev)
} else {
    true
};

if !has_new {
    return; // Nothing new
}

// Only now compute the actual diff (for propagation)
let diff = if let Some(prev) = self.prev_pts.get(&v) {
    let mut d = current.clone();
    d.difference(prev);
    d
} else {
    current.clone()
};
```

This avoids the clone for the common case where nothing changed.

**Step: Implement the chosen optimization, test, benchmark, commit**

After implementing:

```bash
# Format and lint
make fmt && make lint

# Run tests
docker compose run --rm dev sh -c 'cargo nextest run --release -p saf-analysis 2>&1 | tail -5'

# Re-profile to verify improvement
docker compose run --rm dev sh -c 'cargo run --release -p saf-bench --features solver-stats -- cruxbc --compiled-dir tests/benchmarks/cruxbc/.compiled --filter "big/bash" -o /workspace/tests/benchmarks/cruxbc/post-opt1-126.json 2>&1 | tee /tmp/profile-126-post-opt1.txt'
```

---

### Task 6: PTABen Regression Check and Finalize

**Files:**
- Modify: `plans/126-andersen-profiling-optimization.md` (results)
- Modify: `plans/PROGRESS.md`

**Step 1: Run PTABen correctness check**

```bash
docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- ptaben --compiled-dir tests/benchmarks/ptaben/.compiled -o /workspace/tests/benchmarks/ptaben/results-126.json'
```

Expected: 2236 Exact, 69 Unsound (no regression).

**Step 2: Run full CruxBC benchmark (all programs)**

```bash
docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- cruxbc --compiled-dir tests/benchmarks/cruxbc/.compiled --filter "small,big" -o /workspace/tests/benchmarks/cruxbc/results-126.json'
```

Verify no regression on small programs.

**Step 3: Run full test suite**

```bash
make fmt && make lint
docker compose run --rm dev sh -c 'cargo nextest run --release 2>&1 | tail -10'
```

**Step 4: Update this plan with results**

Add a Results section at the top of this plan document recording:
- Profile output from Task 3
- Identified bottlenecks from Task 4
- Optimization applied from Task 5
- Before/after Andersen times
- PTABen regression check results

**Step 5: Update PROGRESS.md**

Add Plan 126 to the Plans Index:
```
| 126 | andersen-profiling-optimization | performance | done |
```

Add Session Log entry summarizing: profile findings, optimization applied, before/after performance.

Update "Next Steps" based on remaining profile data (e.g., if copy handling was 40% and we cut it in half, what's the new #1 bottleneck?).

**Step 6: Format, lint, commit**

```bash
make fmt && make lint
git add plans/126-andersen-profiling-optimization.md plans/PROGRESS.md
git commit -m "docs: Plan 126 results - profile-guided Andersen PTA optimization"
```

---

## Optimization Response Matrix (Reference)

This table maps profile findings to specific code changes. Use after Task 4.

| Profile Finding | Optimization | Files | Expected Impact |
|---|---|---|---|
| `handle_copy` > 40% | Single-element fast path in `BTreePtsSet::union` | `ptsset/btree.rs` | 10-30% of copy time |
| `handle_copy` > 40% | Replace `Vec::to_vec()` with slice reference in handle methods | `solver.rs` | Eliminates SmallVec→Vec allocation |
| `time_diff` > 15% | Early subset check before clone | `solver.rs` | Avoids clone in ~50% of calls |
| `union_changed` < 20% | Track "last changed" timestamp, skip stable values | `solver.rs` | Reduces worklist churn |
| IndexMap hot (worklist > 10%) | FxHashMap/nohash-hasher for u128 keys | `solver.rs`, `constraint_index.rs` | 2-5x faster lookups |
| `find_rep` avg > 2 hops | `&mut self` path compression (no Cell) | `solver.rs` | Amortized O(alpha(n)) |
| `scc_detect` > 5% | Increase Tarjan interval or make adaptive | `solver.rs` | Direct time saving |
| `worklist_ops` > 10% | Replace `BTreeSet<(u32, ValueId)>` with `BinaryHeap` | `solver.rs` | O(log n) vs O(log n) but better constants |

## Rollback Plan

- **Instrumentation:** Simply disable the `solver-stats` feature. Zero code changes needed.
- **Optimizations:** Each optimization is in a separate commit, independently revertible via `git revert`.
