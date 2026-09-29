//! In-process engine switch: [`rebind`].
//!
//! Split from [`crate::memory::binding`] to keep that file within the layout
//! limit; it owns no state of its own and works on the binding cache there.

use std::path::Path;
use std::sync::Arc;

use crate::config::schema::MemorySubsystemConfig;
use crate::core::subsystem::DriverClass;

use super::binding::{for_workspace, MemoryBinding, BINDINGS};

/// Switch a workspace to `new_cfg` in process: evict the bindings the switch
/// invalidates, re-point every context that serves the workspace, bind the new
/// driver and announce it.
///
/// What is evicted: every cached binding of `workspace_dir` that is not a
/// built-in (`Module`/`Null`) bound under exactly `new_cfg`. An `External`
/// binding is evicted even when its config is unchanged, because the thing that
/// changed may be the keychain entry behind `credential_ref`, which the cache
/// key cannot see. Evicted `External` drivers are shut down in the background;
/// the module is never shut down here — it is process-global and a switch back
/// to it must find it running.
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

    for binding in evicted
        .into_iter()
        .filter(|b| b.class() == DriverClass::External)
    {
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            handle.spawn(async move {
                if let Err(e) = binding.provider().shutdown().await {
                    log::debug!("[memory:binding] evicted driver shutdown failed: {e}");
                }
            });
        }
    }

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
