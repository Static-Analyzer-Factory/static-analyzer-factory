//! CFL context for demand-driven and checker traversal.

use saf_core::ids::InstId;

// ---------------------------------------------------------------------------
// CallString (CFL Context)
// ---------------------------------------------------------------------------

/// CFL context for demand-driven traversal.
///
/// Represents the call string (sequence of call sites) used for context-sensitive
/// analysis. Call/return edges must be balanced (matched parentheses in CFL).
///
/// The call string grows unbounded during traversal; CFL matching naturally
/// bounds context via balanced parentheses (returns must match calls).
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct CallString {
    /// Stack of call-site IDs (most recent last).
    sites: Vec<InstId>,
}

impl CallString {
    /// Create an empty call string (same-function context).
    #[must_use]
    pub fn empty() -> Self {
        Self { sites: Vec::new() }
    }

    /// Check if the call string is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.sites.is_empty()
    }

    /// Get the depth (length) of the call string.
    #[must_use]
    pub fn depth(&self) -> usize {
        self.sites.len()
    }

    /// Push a call site onto the call string (entering callee).
    ///
    /// This represents the opening parenthesis ⟨i in CFL.
    #[must_use]
    pub fn push(&self, call_site: InstId) -> Self {
        let mut new_sites = self.sites.clone();
        new_sites.push(call_site);
        Self { sites: new_sites }
    }

    /// Pop the top call site from the call string (exiting callee).
    ///
    /// Returns `Some((new_context, popped_site))` if non-empty, `None` otherwise.
    /// This represents the closing parenthesis ⟩i in CFL.
    ///
    /// # Panics
    ///
    /// This function cannot panic - the `expect` is guarded by the empty check.
    #[must_use]
    pub fn pop(&self) -> Option<(Self, InstId)> {
        if self.sites.is_empty() {
            None
        } else {
            let mut new_sites = self.sites.clone();
            let popped = new_sites.pop().expect("just checked non-empty");
            Some((Self { sites: new_sites }, popped))
        }
    }

    /// Check if the top call site matches a return site.
    ///
    /// Used for CFL matching: a return edge from callee back to caller
    /// should only be taken if the return site matches the top call site.
    #[must_use]
    pub fn matches(&self, return_site: InstId) -> bool {
        self.sites.last() == Some(&return_site)
    }

    /// Peek at the top call site without popping.
    #[must_use]
    pub fn top(&self) -> Option<InstId> {
        self.sites.last().copied()
    }

    /// Get all call sites as a slice.
    #[must_use]
    pub fn sites(&self) -> &[InstId] {
        &self.sites
    }
}

impl std::fmt::Debug for CallString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CallString[")?;
        for (i, site) in self.sites.iter().enumerate() {
            if i > 0 {
                write!(f, " -> ")?;
            }
            write!(f, "{site:?}")?;
        }
        write!(f, "]")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn call_string_empty() {
        let cs = CallString::empty();
        assert!(cs.is_empty());
        assert_eq!(cs.depth(), 0);
        assert!(cs.pop().is_none());
        assert!(cs.top().is_none());
    }

    #[test]
    fn call_string_push_pop() {
        let cs = CallString::empty();
        let site1 = InstId::new(1);
        let site2 = InstId::new(2);

        let cs = cs.push(site1);
        assert_eq!(cs.depth(), 1);
        assert!(!cs.is_empty());
        assert_eq!(cs.top(), Some(site1));

        let cs = cs.push(site2);
        assert_eq!(cs.depth(), 2);
        assert_eq!(cs.top(), Some(site2));

        let (cs, popped) = cs.pop().unwrap();
        assert_eq!(popped, site2);
        assert_eq!(cs.depth(), 1);

        let (cs, popped) = cs.pop().unwrap();
        assert_eq!(popped, site1);
        assert!(cs.is_empty());
    }

    #[test]
    fn call_string_matches() {
        let cs = CallString::empty();
        let site1 = InstId::new(1);
        let site2 = InstId::new(2);

        let cs = cs.push(site1);
        assert!(cs.matches(site1));
        assert!(!cs.matches(site2));

        let cs = cs.push(site2);
        assert!(cs.matches(site2));
        assert!(!cs.matches(site1));
    }

    #[test]
    fn call_string_immutable() {
        let cs = CallString::empty();
        let site = InstId::new(1);

        let cs2 = cs.push(site);
        // Original is unchanged
        assert!(cs.is_empty());
        assert_eq!(cs2.depth(), 1);
    }

    #[test]
    fn call_string_sites() {
        let cs = CallString::empty()
            .push(InstId::new(1))
            .push(InstId::new(2))
            .push(InstId::new(3));

        let sites = cs.sites();
        assert_eq!(sites.len(), 3);
        assert_eq!(sites[0], InstId::new(1));
        assert_eq!(sites[1], InstId::new(2));
        assert_eq!(sites[2], InstId::new(3));
    }
}
