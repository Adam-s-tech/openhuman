//! Python child-process launch helpers.
//!
//! Uses unbuffered stdio (`-u`) by default so line-oriented protocols such as
//! MCP do not stall behind Python's output buffering.

use std::collections::BTreeMap;
use std::path::PathBuf;

/// Launch spec for a Python stdio subprocess.
#[derive(Debug, Clone)]
pub struct PythonLaunchSpec {
    /// Absolute or caller-resolved path to the Python script.
    pub script_path: PathBuf,
    /// Positional arguments forwarded after the script path.
    pub args: Vec<String>,
    /// Optional working directory for the child process.
    pub cwd: Option<PathBuf>,
    /// Extra environment variables to set on the child process.
    pub env: BTreeMap<String, String>,
    /// Whether to pass `-u` for unbuffered stdio. Defaults to `true`.
    pub unbuffered: bool,
}

impl PythonLaunchSpec {
    pub fn new(script_path: PathBuf) -> Self {
        Self {
            script_path,
            args: Vec::new(),
            cwd: None,
            env: BTreeMap::new(),
            unbuffered: true,
        }
    }
}

#[cfg(test)]
#[path = "process_tests.rs"]
mod tests;
