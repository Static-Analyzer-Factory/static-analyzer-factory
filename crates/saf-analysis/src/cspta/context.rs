//! Call-site context type for k-CFA pointer analysis.
//!
//! A `CallSiteContext` is a bounded sequence of call-site instruction IDs
//! representing the call chain that led to the current analysis scope.
//! Empty context corresponds to context-insensitive (global scope).
//! SCC functions use k-1 limited contexts for partial sensitivity.

use saf_core::ids::InstId;

/// A call-site context: sequence of up to k call-site `InstId` values.
///
/// Empty context = context-insensitive (global scope).
/// SCC functions use k-1 limited contexts instead of empty.
/// Push appends a call-site and truncates the oldest if over `k`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CallSiteContext {
    sites: Vec<InstId>,
}

impl CallSiteContext {
    /// Create an empty (context-insensitive) context.
    #[must_use]
    pub fn empty() -> Self {
        Self { sites: Vec::new() }
    }

    /// Push a call-site onto the context, truncating to depth `k`.
    ///
    /// If the resulting context exceeds `k` entries, the oldest
    /// (leftmost) entries are removed by draining the prefix in O(1)
    /// amortised (single `drain` call instead of per-element `remove(0)`).
    #[must_use]
    pub fn push(&self, site: InstId, k: u32) -> Self {
        let mut new_sites = self.sites.clone();
        new_sites.push(site);
        let limit = k as usize;
        if new_sites.len() > limit {
            let excess = new_sites.len() - limit;
            new_sites.drain(..excess);
        }
        Self { sites: new_sites }
    }

    /// Pop the most recent call-site from the context.
    ///
    /// Returns the shortened context and the popped site (if any).
    #[must_use]
    pub fn pop(&self) -> (Self, Option<InstId>) {
        if self.sites.is_empty() {
            return (Self::empty(), None);
        }
        let mut new_sites = self.sites.clone();
        let popped = new_sites.pop();
        (Self { sites: new_sites }, popped)
    }

    /// Check if this is an empty (CI) context.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.sites.is_empty()
    }

    /// Get the context depth (number of call sites).
    #[must_use]
    pub fn len(&self) -> usize {
        self.sites.len()
    }

    /// Get the call sites as a slice.
    #[must_use]
    pub fn sites(&self) -> &[InstId] {
        &self.sites
    }

    /// Truncate the context to at most `max_len` entries, keeping the
    /// most recent (rightmost) call sites.
    ///
    /// Used for k-1 limiting within SCCs: instead of collapsing to the
    /// empty context, retain `k-1` levels of context sensitivity.
    #[must_use]
    pub fn truncate(&self, max_len: usize) -> Self {
        if self.sites.len() <= max_len {
            return self.clone();
        }
        let start = self.sites.len() - max_len;
        Self {
            sites: self.sites[start..].to_vec(),
        }
    }

    /// Export context as hex strings.
    #[must_use]
    pub fn to_hex_vec(&self) -> Vec<String> {
        self.sites.iter().map(|s| s.to_hex()).collect()
    }
}

impl std::fmt::Display for CallSiteContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.sites.is_empty() {
            write!(f, "[]")
        } else {
            let hexes: Vec<String> = self.sites.iter().map(|s| s.to_hex()).collect();
            write!(f, "[{}]", hexes.join(", "))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_context() {
        let ctx = CallSiteContext::empty();
        assert!(ctx.is_empty());
        assert_eq!(ctx.len(), 0);
    }

    #[test]
    fn push_within_k() {
        let ctx = CallSiteContext::empty();
        let site1 = InstId::derive(b"site1");
        let ctx1 = ctx.push(site1, 3);
        assert_eq!(ctx1.len(), 1);
        assert_eq!(ctx1.sites()[0], site1);
    }

    #[test]
    fn push_truncates_at_k() {
        let ctx = CallSiteContext::empty();
        let s1 = InstId::derive(b"s1");
        let s2 = InstId::derive(b"s2");
        let s3 = InstId::derive(b"s3");

        let ctx1 = ctx.push(s1, 2);
        let ctx2 = ctx1.push(s2, 2);
        let ctx3 = ctx2.push(s3, 2);

        // k=2, so only s2 and s3 should remain
        assert_eq!(ctx3.len(), 2);
        assert_eq!(ctx3.sites()[0], s2);
        assert_eq!(ctx3.sites()[1], s3);
    }

    #[test]
    fn push_k1_keeps_only_last() {
        let ctx = CallSiteContext::empty();
        let s1 = InstId::derive(b"s1");
        let s2 = InstId::derive(b"s2");

        let ctx1 = ctx.push(s1, 1);
        assert_eq!(ctx1.len(), 1);
        assert_eq!(ctx1.sites()[0], s1);

        let ctx2 = ctx1.push(s2, 1);
        assert_eq!(ctx2.len(), 1);
        assert_eq!(ctx2.sites()[0], s2);
    }

    #[test]
    fn pop_empty() {
        let ctx = CallSiteContext::empty();
        let (popped, site) = ctx.pop();
        assert!(popped.is_empty());
        assert!(site.is_none());
    }

    #[test]
    fn pop_returns_last() {
        let s1 = InstId::derive(b"s1");
        let s2 = InstId::derive(b"s2");
        let ctx = CallSiteContext::empty().push(s1, 3).push(s2, 3);

        let (shorter, popped) = ctx.pop();
        assert_eq!(popped, Some(s2));
        assert_eq!(shorter.len(), 1);
        assert_eq!(shorter.sites()[0], s1);
    }

    #[test]
    fn ordering_is_deterministic() {
        let s1 = InstId::derive(b"s1");
        let s2 = InstId::derive(b"s2");

        let ctx_a = CallSiteContext::empty().push(s1, 2);
        let ctx_b = CallSiteContext::empty().push(s2, 2);

        // Should have a deterministic ordering
        assert_ne!(ctx_a, ctx_b);
        let a_lt_b = ctx_a < ctx_b;
        let b_lt_a = ctx_b < ctx_a;
        assert!(a_lt_b || b_lt_a, "contexts should be ordered");
    }

    #[test]
    fn equality() {
        let s1 = InstId::derive(b"s1");
        let ctx_a = CallSiteContext::empty().push(s1, 2);
        let ctx_b = CallSiteContext::empty().push(s1, 2);
        assert_eq!(ctx_a, ctx_b);
    }

    #[test]
    fn display_empty() {
        let ctx = CallSiteContext::empty();
        assert_eq!(format!("{ctx}"), "[]");
    }

    #[test]
    fn display_non_empty() {
        let s1 = InstId::derive(b"s1");
        let ctx = CallSiteContext::empty().push(s1, 2);
        let display = format!("{ctx}");
        assert!(display.starts_with('['));
        assert!(display.ends_with(']'));
        assert!(display.contains("0x"));
    }
}
