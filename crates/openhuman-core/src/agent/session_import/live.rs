//! Live dual-write of new session turns into the TinyAgents store.
//!
//! Additive, best-effort, and gated by the `AgentConfig::session_dual_write`
//! **config flag** which **defaults ON** ([`dual_write_enabled`]); the
//! `OPENHUMAN_SESSION_DUAL_WRITE` env var is a **kill switch** — set it to a
//! falsey value (`0`/`false`/`no`/`off`/`disable`) to force the mirror off
//! regardless of config. This mirrors the `OPENHUMAN_APPROVAL_GATE`
//! default-on-with-kill-switch idiom. The legacy `session_raw/*.jsonl`
//! transcript (`session/turn/session_io.rs` → `transcript::write_transcript`)
//! stays the primary and authoritative writer; this module mirrors each
//! *already-persisted* turn into the same store layout the Phase-1 importer
//! produces (`{workspace}/tinyagents_store/{kv,journal}`), reusing
//! [`super::convert`] normalization so live and imported records are
//! shape-identical.
//!
//! Reads stay 100% legacy in this slice — 04.2 flips readers independently,
//! gated on the same flag. A store-write failure here must never fail or alter
//! a chat turn: the caller treats every error as non-fatal (log + swallow), and
//! nothing in this module touches the legacy transcript path.

use std::path::Path;
use std::sync::Arc;

use tinyagents_harness::store::Store;
use tinyagents_session::transcript::import::ops::{open_session_stores, SessionStores};

use tinyagents_session::transcript::SessionTranscript;

use super::convert::{
    build_descriptor, effective_thread_id, journal_messages, sanitize_store_name, stream_name,
};
use super::ops::{open_session_stores, SessionStores};
use super::types::{DescriptorSource, JournalMessage, NS_SESSIONS};

/// Kill-switch env var for the live session-store dual-write. The config flag
/// (`AgentConfig::session_dual_write`) defaults ON; setting this env var to a
/// falsey value forces the mirror OFF regardless of config. See
/// [`dual_write_enabled`].
const DUAL_WRITE_ENV: &str = "OPENHUMAN_SESSION_DUAL_WRITE";

/// Kill-switch env var for the store-backed session shadow read. The config
/// flag (`AgentConfig::session_shadow_reads`) defaults ON since the Phase 2
/// parity soak; setting this env var to a falsey value forces the shadow read
/// OFF even when the flag is ON. It can never force the shadow read ON. See
/// [`shadow_reads_enabled`].
const SHADOW_READ_ENV: &str = "OPENHUMAN_SESSION_SHADOW_READS";

/// Whether `var` is set to a case-insensitive falsey value
/// (`0`/`false`/`no`/`off`/`disable`/`disabled`). Unset — or any non-falsey
/// value — is not a kill. Read live (not cached) so a config reload / env
/// change is honored on the next turn/read.
fn env_kill_switch_engaged(var: &str) -> bool {
    match std::env::var(var) {
        Ok(v) => matches!(
            v.trim().to_ascii_lowercase().as_str(),
            "0" | "false" | "no" | "off" | "disable" | "disabled"
        ),
        Err(_) => false,
    }
}

/// Whether the `OPENHUMAN_SESSION_DUAL_WRITE` kill switch is engaged (set to a
/// falsey value). Unset — or any non-falsey value — leaves the mirror driven by
/// the config flag. Read live (not cached) so a config reload / env change is
/// honored on the next turn.
fn kill_switch_engaged() -> bool {
    env_kill_switch_engaged(DUAL_WRITE_ENV)
}

/// Store-registry name under which the session KV store is registered on each
/// turn's `RunContext.stores` (issue #4249, 04.1). Slash-free so it round-trips
/// the crate `FileStore` name sanitizer. This is a forward-looking,
/// harness-visible handle to the same `tinyagents_store` KV tree the live
/// dual-write mirrors into; readers stay legacy until 04.2.
pub const TINYAGENTS_SESSION_KV_STORE: &str = "openhuman_sessions";

/// Whether the live session-store dual-write is enabled for this turn.
///
/// `config_enabled` is the `AgentConfig::session_dual_write` flag, which
/// **defaults ON**. The `OPENHUMAN_SESSION_DUAL_WRITE` env var is a pure kill
/// switch: an explicit falsey value (case-insensitive
/// `0`/`false`/`no`/`off`/`disable`/`disabled`) forces the mirror OFF regardless
/// of config; otherwise the config flag wins. Read live (never cached) so a
/// config reload / env change is honored on the next turn. This keeps a clean
/// 04.2 seam (reads can flip independently) while making the mirror the default
/// so new turns land in the store without opt-in.
pub fn dual_write_enabled(config_enabled: bool) -> bool {
    let killed = kill_switch_engaged();
    let enabled = config_enabled && !killed;
    log::debug!(
        "[session-store] dual-write decision config_enabled={config_enabled} kill_switch={killed} enabled={enabled}"
    );
    enabled
}

/// Open the session KV store as an `Arc<dyn Store>` for registration on the
/// per-turn `RunContext.stores` under [`TINYAGENTS_SESSION_KV_STORE`], honoring
/// the dual-write flag (config default ON + env kill switch).
///
/// Best-effort: `None` when the dual-write is disabled **or** the config (hence
/// workspace) cannot be resolved. When present it is the exact same
/// `{workspace}/tinyagents_store/kv` `FileStore` the importer and the live
/// dual-write use, so a harness-side reader (04.2+) sees identical records. The
/// journal (`JsonlAppendStore`, an `AppendStore` rather than a `Store`) is not
/// registrable on the `StoreRegistry`; the dual-write opens it directly.
pub async fn session_kv_store() -> Option<Arc<dyn Store>> {
    let cfg = match crate::config::Config::load_or_init().await {
        Ok(cfg) => cfg,
        Err(err) => {
            log::warn!("[session-store] cannot resolve config for store registration: {err:#}");
            return None;
        }
    };
    if !dual_write_enabled(cfg.agent.session_dual_write) {
        log::debug!(
            "[session-store] dual-write disabled; skipping RunContext session-store registration"
        );
        return None;
    }
    let workspace = cfg.workspace_dir;
    let SessionStores { kv, .. } = open_session_stores(&workspace);
    log::debug!(
        "[session-store] opened session kv store for RunContext.stores workspace={}",
        workspace.display()
    );
    Some(Arc::new(kv))
}

// ─────────────────────────────────────────────────────────────────────────────
// Store-backed SHADOW READ (issue #4249, sessions 04.2 phase 2)
//
// Beside the legacy authoritative transcript reader
// (`session/turn/session_io.rs` → `try_load_session_transcript`), read the
// same session's messages back from the crate journal store, normalize both
// sides through the same `convert` machinery the dual-write uses, compare, and
// log divergence. Legacy stays authoritative: this observes + logs only and
// never affects, fails, or slows the authoritative read.
// ─────────────────────────────────────────────────────────────────────────────

/// Whether the store-backed session **shadow read** is enabled for this read.
///
/// `config_enabled` is the `AgentConfig::session_shadow_reads` flag, which
/// **defaults ON** since the Phase 2 parity soak, as `session_dual_write`
/// already did. The
/// `OPENHUMAN_SESSION_SHADOW_READS` env var is a pure kill switch: an explicit
/// falsey value (case-insensitive `0`/`false`/`no`/`off`/`disable`/`disabled`)
/// forces the shadow read OFF regardless of config; it can never force it ON.
/// Read live (never cached) so a config reload / env change is honored on the
/// next read. Mirrors the [`dual_write_enabled`] flag/env idiom exactly: the
/// env var can only ever force OFF, never ON.
pub fn shadow_reads_enabled(config_enabled: bool) -> bool {
    let killed = env_kill_switch_engaged(SHADOW_READ_ENV);
    let enabled = config_enabled && !killed;
    log::debug!(
        "[session_shadow_read] decision config_enabled={config_enabled} kill_switch={killed} enabled={enabled}"
    );
    enabled
}
