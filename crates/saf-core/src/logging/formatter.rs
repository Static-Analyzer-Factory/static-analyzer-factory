//! DSL line formatter for SAF structured debug logging.
//!
//! Produces lines in the format:
//! ```text
//! [module::phase][tag] narrative | key=value key=value
//! ```

/// Format a complete SAF log line.
///
/// - `module`, `phase`, `tag`: the log coordinates
/// - `narrative`: optional human-readable summary (empty string if keys-only)
/// - `kv`: pre-formatted key=value pairs (empty string if narrative-only)
pub fn format_saf_log_line(
    module: &str,
    phase: &str,
    tag: &str,
    narrative: &str,
    kv: &str,
) -> String {
    let mut line = String::with_capacity(128);

    // Prefix: [module::phase][tag]
    line.push('[');
    line.push_str(module);
    line.push_str("::");
    line.push_str(phase);
    line.push_str("][");
    line.push_str(tag);
    line.push(']');

    match (narrative.is_empty(), kv.is_empty()) {
        // Keys only: [mod::phase][tag] | key=val
        (true, false) => {
            line.push_str(" | ");
            line.push_str(kv);
        }
        // Narrative only: [mod::phase][tag] narrative
        (false, true) => {
            line.push(' ');
            line.push_str(narrative);
        }
        // Full form: [mod::phase][tag] narrative | key=val
        (false, false) => {
            line.push(' ');
            line.push_str(narrative);
            line.push_str(" | ");
            line.push_str(kv);
        }
        // Neither (shouldn't happen, but handle gracefully)
        (true, true) => {}
    }

    line
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_form() {
        let line = format_saf_log_line(
            "pta",
            "solve",
            "worklist",
            "pts grew",
            "val=0x1a2b pts_size=3",
        );
        assert_eq!(
            line,
            "[pta::solve][worklist] pts grew | val=0x1a2b pts_size=3"
        );
    }

    #[test]
    fn test_narrative_only() {
        let line = format_saf_log_line("pta", "solve", "convergence", "fixpoint reached", "");
        assert_eq!(line, "[pta::solve][convergence] fixpoint reached");
    }

    #[test]
    fn test_keys_only() {
        let line = format_saf_log_line("pta", "solve", "stats", "", "iter=12 worklist=342");
        assert_eq!(line, "[pta::solve][stats] | iter=12 worklist=342");
    }
}
