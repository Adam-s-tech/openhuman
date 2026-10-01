//! Per-toolkit agent scope preferences, read and written through the bound
//! memory driver.
//!
//! # Why this is host code now
//!
//! The two `composio.{get,set}_user_scopes` handlers used to reach
//! `tinymemory_core::sync::composio::providers::user_scopes::{load_or_default,
//! save}` — engine functions taking a `&MemoryClientRef`, resolved from the
//! host's in-process `global` slot. openhuman#5560 deletes that second engine,
//! so both calls had to move. Neither is a capability a second memory driver
//! would answer differently: they are two rows in the driver's key/value tier,
//! which the contract already exposes as [`MemoryGraph`]. So the *policy* came
//! home and the *storage* went over the bus, exactly as `memory::safety` and
//! `util::redact` did — no contract member was added.
//!
//! # The storage shape is the engine's, byte for byte
//!
//! The namespace, the key derivation and the serialised value below are copied
//! from `tinymemory-core`'s `user_scopes.rs` rather than re-invented, because
//! **this host is not the only reader**. Engine code still consults the same
//! rows to gate tool calls by scope, and it now runs inside the loaded module.
//! A key this host lower-cased differently, or a namespace one character off,
//! would not fail — it would read as "no preference stored" and silently hand
//! the agent the default (`read+write`, no `admin`) while the user's saved
//! choice sat one key away. `user_scopes_storage_shape_matches_the_engine`
//! pins the three constants against that.

use serde_json::Value;

use crate::config::Config;

/// The stored type, still the engine's.
///
/// Named through `integrations::composio::providers` — the host's re-export of
/// the engine's provider vocabulary — rather than redeclared here. `ToolScope`
/// gating, the catalogue and the RPC reply all read this exact struct, and a
/// host-side twin would be a second serde shape over one row. It is the wire
/// shape of both handlers' replies, so it must not be swapped for a look-alike.
pub(crate) use crate::integrations::composio::providers::UserScopePref;

/// KV namespace holding one row per toolkit.
///
/// `tinymemory-core`'s `user_scopes::KV_NAMESPACE`. Deliberately distinct from
/// `composio-sync-state`, so prefs and sync cursors never collide.
const KV_NAMESPACE: &str = "composio-user-scopes";

/// The row key for a toolkit — trimmed and ASCII-lowercased.
///
/// `tinymemory-core`'s `user_scopes::kv_key`. Both RPC handlers reach here
/// through `read_required_non_empty`, which has already rejected an
/// all-whitespace toolkit, so the empty-key arm those engine functions carry
/// cannot be reached from the RPC surface; it is still handled rather than
/// assumed away, because a future caller need not come through that helper.
fn kv_key(toolkit: &str) -> String {
    toolkit.trim().to_ascii_lowercase()
}

/// Persist the scope pref for `toolkit`.
///
/// **This one fails closed.** A write that reported success without storing
/// anything would leave the user looking at a toggle they just moved while the
/// agent kept the old permissions — and the next read, which fails open, would
/// confirm the stale value rather than contradict it. Every failure is an
/// `Err` the RPC surfaces.
///
/// # Errors
///
/// An empty toolkit, an unresolvable driver, a driver with no `Graph` family,
/// or a backend write failure.
pub(crate) async fn save(
    config: &Config,
    toolkit: &str,
    pref: UserScopePref,
) -> Result<(), String> {
    let key = kv_key(toolkit);
    if key.is_empty() {
        return Err("user_scopes: toolkit must not be empty".to_string());
    }
    let value: Value = serde_json::to_value(pref)
        .map_err(|e| format!("[composio][scopes] serialize failed: {e}"))?;

    let binding = crate::memory::binding::for_config(config)
        .map_err(|e| format!("[composio][scopes] memory driver unavailable: {e}"))?;
    let Some(graph) = binding.provider().as_graph() else {
        return Err(format!(
            "[composio][scopes] driver '{}' does not serve Graph, cannot persist pref",
            binding.driver_id()
        ));
    };

    graph
        .kv_put(Some(KV_NAMESPACE), &key, value)
        .await
        .map_err(|e| format!("[composio][scopes] kv_put failed: {e}"))?;

    tracing::info!(
        toolkit = %key,
        read = pref.read,
        write = pref.write,
        admin = pref.admin,
        driver = binding.driver_id(),
        "[composio][scopes] pref saved"
    );
    Ok(())
}

#[cfg(test)]
#[path = "user_scopes_tests.rs"]
mod tests;
