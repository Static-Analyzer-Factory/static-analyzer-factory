//! Benchmark reporting output formats.
//!
//! Supports human-readable tables and JSON output with dual ground truth outcomes.

use crate::SuiteSummary;
use anyhow::Result;
use std::io::Write;

/// Print a human-readable summary table to stdout.
pub fn print_human(summary: &SuiteSummary) {
    print_human_to(summary, &mut std::io::stdout().lock());
}

/// Format a duration as a human-friendly string.
fn format_time(secs: f64) -> String {
    if secs < 0.1 {
        format!("{:.0}ms", secs * 1000.0)
    } else if secs < 60.0 {
        format!("{secs:.1}s")
    } else {
        let mins = (secs / 60.0).floor();
        let rem = secs - mins * 60.0;
        format!("{mins:.0}m{rem:.0}s")
    }
}

/// Print a human-readable summary table to an arbitrary writer.
// NOTE: This function formats the entire benchmark report (header, category
// table, failure details, expected-fail improvements). Splitting by section
// would scatter related formatting logic across many small functions.
#[allow(clippy::too_many_lines)]
pub fn print_human_to(summary: &SuiteSummary, w: &mut dyn Write) {
    // Helper macro to avoid repeating `let _ =` on every writeln
    macro_rules! out {
        ($($arg:tt)*) => { let _ = writeln!(w, $($arg)*); };
        () => { let _ = writeln!(w); };
    }

    let total_outcomes =
        summary.exact + summary.sound + summary.to_verify + summary.unsound + summary.skip;

    out!();
    out!(
        "  {} Benchmark Results  ({} files, {} checks, {})",
        summary.suite,
        summary.total,
        total_outcomes,
        format_time(summary.timing_secs)
    );
    out!();

    // Category table
    let w_cat = 26;
    let w_col = 7;
    let w_time = 8;
    let rule_len = w_cat + w_col * 6 + w_time + 3;

    out!(
        "  {:<w_cat$} {:>w_col$} {:>w_col$} {:>w_col$} {:>w_col$} {:>w_col$} {:>w_col$}  {:>w_time$}",
        "Category",
        "Exact",
        "Sound",
        "Verify",
        "Unsnd",
        "Skip",
        "Total",
        "Time",
    );
    out!("  {}", "\u{2500}".repeat(rule_len));

    // Category rows sorted by timing descending (slowest first)
    let mut sorted_cats: Vec<_> = summary.by_category.iter().collect();
    sorted_cats.sort_by(|a, b| {
        b.timing_secs
            .partial_cmp(&a.timing_secs)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    for cat in &sorted_cats {
        let total = cat.exact + cat.sound + cat.to_verify + cat.unsound + cat.skip;
        let time_str = format_time(cat.timing_secs);

        // Highlight unsound categories
        let marker = if cat.unsound > 0 { "!" } else { " " };

        out!(
            " {marker}{:<w_cat$} {:>w_col$} {:>w_col$} {:>w_col$} {:>w_col$} {:>w_col$} {:>w_col$}  {:>w_time$}",
            truncate(&cat.category, w_cat),
            cat.exact,
            cat.sound,
            cat.to_verify,
            cat.unsound,
            cat.skip,
            total,
            time_str,
        );
    }

    out!("  {}", "\u{2500}".repeat(rule_len));
    out!(
        "  {:<w_cat$} {:>w_col$} {:>w_col$} {:>w_col$} {:>w_col$} {:>w_col$} {:>w_col$}  {:>w_time$}",
        "TOTAL",
        summary.exact,
        summary.sound,
        summary.to_verify,
        summary.unsound,
        summary.skip,
        total_outcomes,
        format_time(summary.timing_secs),
    );
    out!();

    // EXPECTEDFAIL summary (known imprecision oracles)
    if summary.expectedfail_total > 0 {
        out!(
            "  Expected-imprecision: {} / {} oracles correctly resolved",
            summary.expectedfail_exact,
            summary.expectedfail_total
        );
        if !summary.expectedfail_details.is_empty() {
            for detail in &summary.expectedfail_details {
                out!("    {} - {}", detail.file, detail.result);
            }
        }
        out!();
    }

    // Unsound section (definite failures)
    if !summary.unsound_details.is_empty() {
        out!("  Unsound ({} issues):", summary.unsound_details.len());
        for (i, failure) in summary.unsound_details.iter().enumerate() {
            if i >= 10 {
                out!("    ... and {} more", summary.unsound_details.len() - 10);
                break;
            }
            out!(
                "    {} : floor={}, got={}",
                failure.file,
                failure.expected,
                failure.actual
            );
        }
        out!();
    }

    // To-verify section
    if !summary.to_verify_details.is_empty() {
        out!(
            "  To Verify ({} items - SAF more precise than baseline):",
            summary.to_verify_details.len()
        );
        for (i, detail) in summary.to_verify_details.iter().enumerate() {
            if i >= 5 {
                out!("    ... and {} more", summary.to_verify_details.len() - 5);
                break;
            }
            out!(
                "    {} : floor={}, got={}",
                detail.file,
                detail.floor,
                detail.actual
            );
        }
        out!();
    }

    // Final verdict
    if summary.unsound == 0 {
        out!("  Result: PASS");
    } else {
        out!(
            "  Result: UNSOUND ({} failures need investigation)",
            summary.unsound
        );
    }
    out!();
}

/// Print JSON output.
///
/// # Errors
///
/// Returns an error if JSON serialization fails.
pub fn print_json(summary: &SuiteSummary) -> Result<()> {
    let json = serde_json::to_string_pretty(summary)?;
    println!("{json}");
    Ok(())
}

/// Truncate a string to max length with ellipsis.
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else if max <= 3 {
        s[..max].to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}
