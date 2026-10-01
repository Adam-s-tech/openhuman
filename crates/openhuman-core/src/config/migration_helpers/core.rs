use crate::config::Config;
use crate::memory::Memory;
use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;

pub use tinymemory_import::{MigrationReport, MigrationStats};

/// Import an OpenClaw workspace into this host's memory.
///
/// The importer is `tinymemory-import`'s; the host supplies the target
/// workspace and [`target_memory_backend`], which it opens only once there is
/// something to write.
pub async fn migrate_openclaw_memory(
    config: &Config,
    source_workspace: Option<PathBuf>,
    dry_run: bool,
) -> Result<MigrationReport> {
    tinymemory_import::migrate_openclaw_memory(
        &config.workspace_dir,
        source_workspace,
        dry_run,
        || target_memory_backend(config),
    )
    .await
}

/// Import a Hermes workspace into this host's memory. See
/// [`migrate_openclaw_memory`].
pub async fn migrate_hermes_memory(
    config: &Config,
    source_workspace: Option<PathBuf>,
    dry_run: bool,
) -> Result<MigrationReport> {
    tinymemory_import::migrate_hermes_memory(
        &config.workspace_dir,
        source_workspace,
        dry_run,
        || target_memory_backend(config),
    )
    .await
}

/// The memory the import writes into.
///
/// The bound driver, through [`DriverMemory`] — the same store the engine
/// constructor this replaced opened. That equivalence is checked rather than
/// assumed: `create_memory_for_migration` bottoms out in
/// `create_memory_full(..., workspace_dir, "memory")`, and
/// `binding::for_workspace` resolves `for_subtree(workspace_dir, "memory")`.
/// Same workspace, same subtree.
///
/// # Why the capture budget does not bite here
///
/// This note used to say the contract had no door for an import, because
/// `MemoryCore::store` is reached through `MemoryGuard`, which applies
/// `MemoryHooksConfig::capture_max_chars` to every body — and an imported
/// `MEMORY.md` entry is routinely longer than that, so routing an import
/// through the guard would silently truncate the user's own memories.
///
/// The budget is real, and it is enforced host-side in
/// `memory::guard::policy`. What the note missed is that `DriverMemory` does
/// not go through the guard at all: it wraps `binding.provider()`, which is the
/// **unguarded** driver — the binding keeps the guarded one separately behind
/// `binding.guard()`. So an import writes full bodies here exactly as the
/// engine constructor did, and no policy has to be special-cased to allow it.
///
/// # A null driver is refused, not imported into
///
/// `binding` answers the null driver in three situations: `[subsystems.memory]
/// driver = "null"` is configured; a configured driver failed to bind and fell
/// back; and — the one this note used to miss — a driver was *admitted* as a
/// module but this build has no module to bind, so `binding::module_provider`
/// substituted the null provider under `#[cfg(not(feature = "modules"))]`.
/// In all three, every write below is discarded. An import that reports
/// "migrated N entries" having written none is silent data loss of the worst
/// kind — the source workspace may be deleted on the strength of that report,
/// and the user only discovers it later, with nothing left to re-run against.
/// So this refuses up front and names which of the three it was.
///
/// Telling the third case apart needs what was *admitted*, not what was bound.
/// `admit` is pure config and never saw the feature flag, so it answers
/// `Module` for the configured `tinycortex`; `module_provider` then binds the
/// null provider, and the binding reports `class = Null` with **no**
/// `fallback`, since nothing refused. So `admitted != bound` is the signal, and
/// it is the only one that works: `driver_id` alone does not separate this from
/// a driver deliberately given `class = "null"`, because `admit` accepts a
/// non-built-in id with that class (the `built_in_class` check is skipped for
/// ids that are not built in) and returns it verbatim. Keying on the id would
/// tell a user with a working modules build that their `modules` feature was
/// off — the same class of misdirection this whole note exists to prevent.
///
/// Keying on the class alone was the original bug: it put the modules-off case
/// in the configured-null arm and told a user whose config says `driver =
/// "tinycortex"` that they had set it to `"null"`, sending them to edit a line
/// that already said the opposite.
fn target_memory_backend(config: &Config) -> Result<Arc<dyn Memory>> {
    let binding = crate::memory::binding::for_config(config)
        .map_err(|e| anyhow::anyhow!("bind memory for migration import: {e}"))?;
    if binding.class() == crate::core::subsystem::DriverClass::Null {
        let admitted = crate::memory::binding::admit(&config.subsystems.memory).ok();
        let because = match (binding.fallback(), admitted) {
            (Some(fallback), _) => format!(
                "driver '{}' refused to bind: {}",
                fallback.configured_driver, fallback.reason
            ),
            // Admitted as the null driver: the user asked for no memory.
            (None, Some((id, crate::core::subsystem::DriverClass::Null)))
                if id == tinymemory_api::null::NULL_DRIVER_ID =>
            {
                "memory is disabled by configuration ([subsystems.memory] driver = \"null\")"
                    .to_string()
            }
            (None, Some((id, crate::core::subsystem::DriverClass::Null))) => format!(
                "driver '{id}' is configured with class \"null\" under \
                 [subsystems.memory.drivers.{id}], so every write is discarded"
            ),
            // Admitted as something that can hold data, bound to the null
            // provider anyway — the module substitution.
            (None, Some((id, _))) => format!(
                "driver '{id}' is configured, but this build has no memory module \
                 compiled in (the 'modules' feature is off), so nothing can be written"
            ),
            // `admit` refused: `build` records that as a fallback, so this is
            // unreachable. Named rather than merged into an arm that would
            // state a cause this branch cannot actually establish.
            (None, None) => format!(
                "driver '{}' bound the null provider and the reason was not recorded",
                binding.driver_id()
            ),
        };
        anyhow::bail!(
            "refusing to import memory into the null driver — {because}. \
             Nothing was imported; the source workspace is untouched."
        );
    }
    crate::agent::experience::ops::DriverMemory::for_config(config)
        .map_err(|e| anyhow::anyhow!("bind memory for migration import: {e}"))
}
