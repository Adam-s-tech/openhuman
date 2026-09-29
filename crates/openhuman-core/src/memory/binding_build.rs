//! Building one binding: admission, provider construction, and the loud
//! fallback. Split from [`crate::memory::binding`] for the layout limit.

use std::path::Path;
use std::sync::Arc;

use crate::config::schema::MemorySubsystemConfig;
use crate::core::subsystem::DriverClass;
use tinymemory_api::null::{NullMemoryProvider, NULL_DRIVER_ID};

use super::api::provider::MemoryProvider;
use super::binding::{
    admit, bind_provider, module_provider, FallbackReason, MemoryBinding, TRANSIENT_RETRY_AFTER,
};

/// Construct the provider an admitted driver binds.
///
/// `Null` and `Module` are the two built-ins; `External` goes through the
/// engine factory (`memory-remote`). A construction failure is a
/// [`FallbackReason`] like an admission failure — the slot is never left empty.
fn construct(
    workspace_dir: &Path,
    memory_subdir: &str,
    cfg: &MemorySubsystemConfig,
    driver_id: &str,
    class: DriverClass,
) -> Result<(Arc<dyn MemoryProvider>, DriverClass), FallbackReason> {
    match class {
        DriverClass::Null => Ok((Arc::new(NullMemoryProvider::new()), DriverClass::Null)),
        #[cfg(feature = "memory-remote")]
        DriverClass::External => {
            use super::binding_remote::{build_engine, configured_api_url, EngineTarget};
            let target = EngineTarget::from_entry(driver_id, cfg.drivers.get(driver_id));
            let api_url = configured_api_url(workspace_dir);
            build_engine(workspace_dir, &api_url, &target)
                .map(|provider| (provider, DriverClass::External))
                .map_err(|reason| FallbackReason {
                    configured_driver: driver_id.to_string(),
                    reason,
                })
        }
        _ => {
            let _ = (cfg, driver_id);
            Ok(module_provider(workspace_dir, memory_subdir))
        }
    }
}

/// Build the binding for a workspace. Infallible by design: an inadmissible
/// driver falls back to the placeholder rather than leaving the slot empty
/// (kernel.md §3.7 — "logged loudly, surfaced in status, never silent").
pub(super) fn build(
    workspace_dir: &Path,
    memory_subdir: &str,
    cfg: &MemorySubsystemConfig,
) -> MemoryBinding {
    // `transient` is true only when an admitted External driver failed to
    // construct: that can heal (keychain unlocked, transport installed), so the
    // fallback must not be pinned in the cache. An admission refusal is
    // deterministic for the config and stays cached.
    let mut transient = false;
    let resolved = admit(cfg).and_then(|(driver_id, class)| {
        construct(workspace_dir, memory_subdir, cfg, &driver_id, class)
            .map(|(provider, reported)| (driver_id, provider, reported))
            .inspect_err(|_| transient = class == DriverClass::External)
    });
    match resolved {
        Ok((driver_id, provider, reported_class)) => {
            let binding = bind_provider(
                provider,
                driver_id,
                memory_subdir.to_string(),
                reported_class,
                None,
            );
            log::info!(
                "[memory:binding] workspace={} bound driver='{}' class={} capabilities=[{}]",
                workspace_dir.display(),
                binding.driver_id(),
                binding.class(),
                binding
                    .capabilities()
                    .iter()
                    .map(|c| c.as_str())
                    .collect::<Vec<_>>()
                    .join(",")
            );
            binding
        }
        Err(fallback) => {
            log::warn!(
                "[memory:binding] workspace={} driver '{}' refused to bind ({}); \
                 falling back to '{NULL_DRIVER_ID}' — memory writes are DISCARDED this run",
                workspace_dir.display(),
                fallback.configured_driver,
                fallback.reason
            );
            // Sync, and a no-op when the bus is not yet initialized, so this is
            // safe to call pre-boot with no `#[cfg(test)]` guard.
            crate::core::bus::BUS.publish(
                crate::core::events::DomainEvent::MemoryDriverBindFailed {
                    configured_driver: fallback.configured_driver.clone(),
                    bound_driver: NULL_DRIVER_ID.to_string(),
                    reason: fallback.reason.clone(),
                },
            );
            let binding = bind_provider(
                Arc::new(NullMemoryProvider::new()),
                NULL_DRIVER_ID.to_string(),
                memory_subdir.to_string(),
                DriverClass::Null,
                Some(fallback),
            );
            if transient {
                binding.retry_after(TRANSIENT_RETRY_AFTER)
            } else {
                binding
            }
        }
    }
}
