//! JSON-RPC / CLI controller surface for data migration.

use std::path::PathBuf;

use crate::config::migration_helpers::{self, MigrationReport};
use crate::config::Config;
use crate::core::Outcome;

pub async fn migrate_openclaw(
    config: &Config,
    source_workspace: Option<PathBuf>,
    dry_run: bool,
) -> Result<Outcome<MigrationReport>, String> {
    let report = migration_helpers::migrate_openclaw_memory(config, source_workspace, dry_run)
        .await
        .map_err(|e| e.to_string())?;
    Ok(Outcome::single_log(report, "migration completed"))
}

pub async fn migrate_hermes(
    config: &Config,
    source_workspace: Option<PathBuf>,
    dry_run: bool,
) -> Result<Outcome<MigrationReport>, String> {
    let report = migration_helpers::migrate_hermes_memory(config, source_workspace, dry_run)
        .await
        .map_err(|e| e.to_string())?;
    Ok(Outcome::single_log(report, "migration completed"))
}

#[cfg(test)]
#[path = "ops_tests.rs"]
mod tests;
