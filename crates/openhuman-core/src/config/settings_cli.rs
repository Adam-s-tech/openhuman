//! Settings “section” views for the core CLI (slice full config JSON by area).

use serde_json::json;

/// Fields matching the config snapshot payload shape used by RPC/CLI.
#[derive(Debug, Clone)]
pub struct ConfigSnapshotFields {
    pub config: serde_json::Value,
    pub workspace_dir: String,
    pub config_path: String,
}

#[cfg(test)]
#[path = "settings_cli_tests.rs"]
mod tests;
