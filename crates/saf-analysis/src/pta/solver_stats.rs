//! Solver profiling statistics.
//!
//! Accumulates timing and counters during Andersen PTA solving to identify
//! performance bottlenecks. Always compiled in; output controlled at runtime
//! via `saf_log!` (the `SAF_LOG` environment variable).

use std::time::Duration;

use saf_core::saf_log;

use crate::timer::Timer;

use serde::{Deserialize, Serialize};

/// Accumulated profiling statistics for a single solver run.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
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
    /// Time computing diffs in `process_value` (clone + difference + `is_empty`).
    pub time_diff: Duration,
    /// Time in `detect_and_collapse_cycles` (periodic Tarjan).
    pub time_scc: Duration,
    /// Time in `check_pending_cycles` (LCD).
    pub time_lcd: Duration,
    /// Time in worklist insert/pop operations.
    pub time_worklist: Duration,
    /// Total wall-clock time of all `process_value` calls (to find gap vs handler sum).
    pub time_process_value_total: Duration,
    /// Time cloning current pts into `prev_pts` in `process_value`.
    pub time_prev_pts_clone: Duration,
    /// Total wall-clock time of all `process_location` calls.
    pub time_process_location: Duration,

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
    /// Total `union` calls (on `PtsSet`).
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
    /// Times `process_location` returned early because `load_loc_index` had no entries.
    pub proc_loc_early_exits: u64,
    /// Times `process_location` was a first visit (no prev_loc_pts entry).
    pub proc_loc_first_visits: u64,
    /// Times `process_location` returned early due to empty diff.
    pub proc_loc_diff_empty: u64,
}

impl SolverStats {
    /// Start a timing section. Returns a `Timer` to pass to `end_section`.
    #[inline]
    pub fn start_section() -> Timer {
        Timer::now()
    }

    /// End a timing section and accumulate into the given duration field.
    #[inline]
    #[allow(clippy::needless_pass_by_value)]
    pub fn end_section(start: Timer, accum: &mut Duration) {
        *accum += start.elapsed();
    }

    /// Print a human-readable profile summary via `saf_log!`.
    // Timing and count variables (copy_c/load_c/store_c/gep_c, *_secs) are
    // intentionally named for readability in the profiling output.
    #[allow(clippy::similar_names, clippy::cast_precision_loss)]
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

        let empty_diff_skip_pct = if self.process_value_calls > 0 {
            self.empty_diff_skips as f64 / self.process_value_calls as f64 * 100.0
        } else {
            0.0
        };

        // --- Overview ---
        saf_log!(pta::solve, stats, label;
            copy_constraints = copy_c,
            load_constraints = load_c,
            store_constraints = store_c,
            gep_constraints = gep_c,
            value_pops = self.value_pops,
            loc_pops = self.loc_pops,
            process_value_calls = self.process_value_calls,
            empty_diff_skips = self.empty_diff_skips,
            empty_diff_skip_pct = empty_diff_skip_pct,
        );

        // --- Phase timing ---
        saf_log!(pta::solve, stats, "timing";
            copy = self.time_copy,
            load = self.time_load,
            store = self.time_store,
            gep = self.time_gep,
            diff = self.time_diff,
            scc = self.time_scc,
            lcd = self.time_lcd,
            worklist = self.time_worklist,
            total = total_time,
            copy_indices_processed = self.copy_indices_processed,
            union_calls = self.union_calls,
            union_changed = self.union_changed,
            load_locs_iterated = self.load_locs_iterated,
            store_locs_iterated = self.store_locs_iterated,
            gep_locs_iterated = self.gep_locs_iterated,
            scc_invocations = self.scc_invocations,
            scc_merges = self.scc_merges,
            lcd_invocations = self.lcd_invocations,
            lcd_merges = self.lcd_merges,
        );

        // --- Deep instrumentation ---
        let handler_sum_secs = total_time.as_secs_f64();
        let pv_total_secs = self.time_process_value_total.as_secs_f64();
        let prev_clone_secs = self.time_prev_pts_clone.as_secs_f64();
        let proc_loc_secs = self.time_process_location.as_secs_f64();
        let pv_overhead_secs = pv_total_secs - handler_sum_secs - prev_clone_secs;

        saf_log!(pta::solve, stats, "deep instrumentation";
            process_value_total = self.time_process_value_total,
            handler_sum_secs = handler_sum_secs,
            prev_pts_clone = self.time_prev_pts_clone,
            process_value_overhead_secs = pv_overhead_secs,
            process_location = self.time_process_location,
            process_location_secs = proc_loc_secs,
            proc_loc_early_exits = self.proc_loc_early_exits,
            proc_loc_first_visits = self.proc_loc_first_visits,
            proc_loc_diff_empty = self.proc_loc_diff_empty,
        );

        // --- find_rep stats ---
        let find_rep_avg_hops = if self.find_rep_calls > 0 {
            self.find_rep_total_hops as f64 / self.find_rep_calls as f64
        } else {
            0.0
        };

        saf_log!(pta::solve, stats, "find_rep";
            find_rep_calls = self.find_rep_calls,
            find_rep_total_hops = self.find_rep_total_hops,
            find_rep_max_chain = self.find_rep_max_chain,
            find_rep_avg_hops = find_rep_avg_hops,
        );
    }
}
