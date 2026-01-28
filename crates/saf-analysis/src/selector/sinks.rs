//! Predefined sink selectors for common taint sinks.

use super::Selector;

/// Common sink selector presets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SinkSelector {
    /// Arguments to specific functions.
    FunctionArgs(Vec<(String, Option<u32>)>),
    /// Return statements in specific functions.
    FunctionReturns(Vec<String>),
    /// Custom selectors.
    Custom(Vec<Selector>),
}

impl SinkSelector {
    /// Convert to a list of selectors.
    #[must_use]
    pub fn to_selectors(&self) -> Vec<Selector> {
        match self {
            Self::FunctionArgs(args) => args
                .iter()
                .map(|(f, idx)| Selector::arg_to(f.as_str(), *idx))
                .collect(),
            Self::FunctionReturns(funcs) => funcs
                .iter()
                .map(|f| Selector::function_return(f.as_str()))
                .collect(),
            Self::Custom(selectors) => selectors.clone(),
        }
    }

    /// SQL injection sinks (database query functions).
    #[must_use]
    pub fn sql_injection() -> Self {
        Self::FunctionArgs(vec![
            ("sqlite3_exec".to_string(), Some(1)), // SQL string
            ("mysql_query".to_string(), Some(1)),  // SQL string
            ("mysql_real_query".to_string(), Some(1)),
            ("PQexec".to_string(), Some(1)), // PostgreSQL
            ("PQexecParams".to_string(), Some(1)),
        ])
    }

    /// Command injection sinks (shell/exec functions).
    #[must_use]
    pub fn command_injection() -> Self {
        Self::FunctionArgs(vec![
            ("system".to_string(), Some(0)),
            ("popen".to_string(), Some(0)),
            ("execl".to_string(), Some(0)),
            ("execle".to_string(), Some(0)),
            ("execlp".to_string(), Some(0)),
            ("execv".to_string(), Some(0)),
            ("execve".to_string(), Some(0)),
            ("execvp".to_string(), Some(0)),
        ])
    }

    /// Path traversal sinks (file operations).
    #[must_use]
    pub fn path_traversal() -> Self {
        Self::FunctionArgs(vec![
            ("fopen".to_string(), Some(0)),
            ("open".to_string(), Some(0)),
            ("creat".to_string(), Some(0)),
            ("openat".to_string(), Some(1)),
            ("access".to_string(), Some(0)),
            ("stat".to_string(), Some(0)),
            ("lstat".to_string(), Some(0)),
            ("unlink".to_string(), Some(0)),
            ("remove".to_string(), Some(0)),
            ("rename".to_string(), None), // Both args
            ("mkdir".to_string(), Some(0)),
            ("rmdir".to_string(), Some(0)),
        ])
    }

    /// Format string sinks.
    #[must_use]
    pub fn format_string() -> Self {
        Self::FunctionArgs(vec![
            ("printf".to_string(), Some(0)),
            ("fprintf".to_string(), Some(1)),
            ("sprintf".to_string(), Some(1)),
            ("snprintf".to_string(), Some(2)),
            ("vprintf".to_string(), Some(0)),
            ("vfprintf".to_string(), Some(1)),
            ("vsprintf".to_string(), Some(1)),
            ("vsnprintf".to_string(), Some(2)),
            ("syslog".to_string(), Some(1)),
        ])
    }

    /// Buffer overflow sinks (unsafe memory operations).
    #[must_use]
    pub fn buffer_overflow() -> Self {
        Self::FunctionArgs(vec![
            ("strcpy".to_string(), Some(1)),  // Source
            ("strcat".to_string(), Some(1)),  // Source
            ("gets".to_string(), Some(0)),    // Buffer
            ("sprintf".to_string(), Some(0)), // Buffer
            ("memcpy".to_string(), None),     // All args relevant
            ("memmove".to_string(), None),
        ])
    }

    /// Network output sinks.
    #[must_use]
    pub fn network_output() -> Self {
        Self::FunctionArgs(vec![
            ("send".to_string(), Some(1)),
            ("sendto".to_string(), Some(1)),
            ("sendmsg".to_string(), Some(1)),
            ("write".to_string(), Some(1)), // When used on sockets
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn function_args_to_selectors() {
        let sel = SinkSelector::FunctionArgs(vec![
            ("system".to_string(), Some(0)),
            ("exec".to_string(), None),
        ]);
        let selectors = sel.to_selectors();
        assert_eq!(selectors.len(), 2);
    }

    #[test]
    fn sql_injection_sinks() {
        let sel = SinkSelector::sql_injection();
        let selectors = sel.to_selectors();
        assert!(!selectors.is_empty());
        // Should include sqlite3_exec
        assert!(selectors.iter().any(|s| matches!(
            s,
            Selector::ArgTo { callee, index } if callee == "sqlite3_exec" && *index == Some(1)
        )));
    }

    #[test]
    fn command_injection_sinks() {
        let sel = SinkSelector::command_injection();
        let selectors = sel.to_selectors();
        assert!(!selectors.is_empty());
        // Should include system
        assert!(selectors.iter().any(|s| matches!(
            s,
            Selector::ArgTo { callee, .. } if callee == "system"
        )));
    }

    #[test]
    fn path_traversal_sinks() {
        let sel = SinkSelector::path_traversal();
        let selectors = sel.to_selectors();
        assert!(!selectors.is_empty());
    }

    #[test]
    fn format_string_sinks() {
        let sel = SinkSelector::format_string();
        let selectors = sel.to_selectors();
        assert!(!selectors.is_empty());
    }
}
