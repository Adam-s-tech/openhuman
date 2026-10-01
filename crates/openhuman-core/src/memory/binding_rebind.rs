//! In-process engine switch, [`rebind`], and the binding a workspace resolves
//! to after one, [`current_for`].
//!
//! Split from [`crate::memory::binding`] to keep that file within the layout
//! limit; it owns no state of its own and works on the binding cache there.

use std::path::Path;
use std::sync::Arc;

use crate::config::schema::MemorySubsystemConfig;
use crate::core::subsystem::DriverClass;

use super::binding::{for_workspace, MemoryBinding, BINDINGS};

/// The binding `workspace_dir`'s shared memory tree resolves to now.
///
/// For a caller holding a config from earlier, which may name an engine
/// switched away from since: binding that config again would rebuild the
/// evicted driver. A switch ([`rebind`]) evicts every other binding of the
/// workspace and binds the new engine, so after one the cache holds a single
/// binding for it, under the config switched to. Before any switch that is the
/// one boot bound. That config is resolved again through [`for_workspace`]
/// rather than the cached binding returned, so a transient fallback is retried
/// once its backoff has passed. `cfg` decides only when nothing is cached for
/// the workspace, or when bindings under more than one config are cached and
/// none of them can be told current.
///
/// # Errors
///
/// As [`for_workspace`].
pub fn current_for(
    workspace_dir: &Path,
    cfg: &MemorySubsystemConfig,
) -> Result<Arc<MemoryBinding>, String> {
    let bound = bound_config(workspace_dir);
    for_workspace(workspace_dir, bound.as_ref().unwrap_or(cfg))
}

/// The config of the one binding cached for `workspace_dir`'s shared tree.
fn bound_config(workspace_dir: &Path) -> Option<MemorySubsystemConfig> {
    let cache = BINDINGS.get()?;
    let map = cache.read().ok()?;
    let mut configs = map
        .keys()
        .filter(|(dir, subdir, _)| dir == workspace_dir && subdir == "memory")
        .map(|(_, _, cfg)| cfg);
    let bound = configs.next()?.clone();
    if configs.next().is_some() {
        log::debug!(
            "[memory:binding] workspace={} has bindings under more than one config; none is current",
            workspace_dir.display()
        );
        return None;
    }
    Some(bound)
}

/// Switch a workspace to `new_cfg` in process: evict the bindings the switch
/// invalidates, re-point every context that serves the workspace, bind the new
/// driver and announce it.
///
/// What is evicted: every cached binding of `workspace_dir` that is not a
/// built-in (`Module`/`Null`) bound under exactly `new_cfg`. An `External`
/// binding is evicted even when its config is unchanged, because the thing that
/// changed may be the keychain entry behind `credential_ref`, which the cache
/// key cannot see. Evicted drivers are not shut down: remote providers have no
/// teardown of their own (the trait default is a no-op) and may still be held by
/// a running session, which releases them on drop. The module is process-global
/// and a switch back to it must find it running.
///
/// Callers that already hold a resolved `Arc<MemoryBinding>` (a running agent
/// session, the archivist) keep it until they resolve again; only new
/// resolutions see the new engine.
///
/// # Errors
///
/// Lock poisoning, or memory shutting down. A driver that cannot bind is not an
/// error — it falls back, per kernel.md §3.7, and the returned binding says so.
pub fn rebind(
    workspace_dir: &Path,
    from: &str,
    new_cfg: &MemorySubsystemConfig,
) -> Result<Arc<MemoryBinding>, String> {
    let evicted: Vec<Arc<MemoryBinding>> = {
        let cache = BINDINGS.get_or_init(Default::default);
        let mut map = cache
            .write()
            .map_err(|e| format!("[memory:binding] cache write lock poisoned: {e}"))?;
        let mut evicted = Vec::new();
        map.retain(|(dir, _subdir, cfg), binding| {
            if dir != workspace_dir {
                return true;
            }
            let builtin = matches!(binding.class(), DriverClass::Module | DriverClass::Null);
            if builtin && cfg == new_cfg {
                return true;
            }
            evicted.push(Arc::clone(binding));
            false
        });
        evicted
    };

    for ctx in [
        crate::core::runtime::context::CoreContext::current(),
        crate::core::runtime::context::CoreContext::default_context(),
    ]
    .into_iter()
    .flatten()
    {
        ctx.set_memory_subsystem(workspace_dir, new_cfg.clone())?;
    }

    // No explicit `shutdown()` on the evicted drivers: the remote adapters keep
    // the trait's default (a no-op) and hold nothing but an HTTP client, and a
    // session or the archivist may still be using the old binding. Dropping our
    // reference lets each one release when its last holder goes.
    drop(evicted);

    let binding = for_workspace(workspace_dir, new_cfg)?;
    log::info!(
        "[memory:binding] rebound workspace={} from='{}' to='{}' bound='{}' class={}",
        workspace_dir.display(),
        from,
        new_cfg.driver,
        binding.driver_id(),
        binding.class()
    );
    crate::core::bus::BUS.publish(crate::core::events::DomainEvent::MemoryDriverChanged {
        from: from.to_string(),
        to: new_cfg.driver.clone(),
    });
    Ok(binding)
}
