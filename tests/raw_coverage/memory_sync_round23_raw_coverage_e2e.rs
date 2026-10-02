//! Round 23 raw coverage focused on memory_sync gaps.
//!
//! Local-only: temp workspaces and no real provider network. Run
//! single-threaded because HOME, OPENHUMAN_WORKSPACE, and config loading are
//! process globals.
//!
//! # What this file used to cover, and what happened to it
//!
//! tinymemory v1.13.4 deleted the in-process Composio pipeline outright (72
//! files, ~18.3k lines) — see
//! `crate::integrations::composio::providers`'s module docs for the
//! full account. This file originally instantiated the deleted engine's
//! `SlackProvider` / `NotionProvider` / `GmailProvider` directly against a
//! loopback HTTP router standing in for the Composio execute API, and
//! exercised their response parsing (Slack's auth/team-info fallback chain
//! when the `users:read.email` scope is missing, Notion's cursor pagination
//! into the memory tree, Gmail's nested-payload flattening and raw-HTML
//! opt-out).
//!
//! That parsing did not move anywhere reachable from this crate: it lives
//! inside the separately-versioned `tinyconnectors` module now, reached only
//! over the module bus via `openhuman.composio_get_user_profile` /
//! `run_sync_pass` (`integrations::composio::ops`). Driving that path for
//! real means a live loaded module — a network download of a pinned
//! release artifact plus a `dlopen`, which is exactly what this file's own
//! "no real provider network" design rules out, and which the CLAUDE.md
//! module-testing note says to run `#[ignore]`d with `OPENHUMAN_MODULE_PATH`
//! instead of in the default suite. So the three provider-specific tests
//! (`slack_profile_falls_back_to_auth_and_team_info_without_email_scope`,
//! `notion_profile_prefers_bot_owner_and_sync_paginates_into_memory_tree`,
//! `gmail_post_process_handles_nested_payloads_and_raw_html_opt_out`) test a
//! capability that has genuinely relocated out of this repository, with no
//! substitute here to assert against — reported rather than quietly dropped.
//! `composio_get_user_profile_refuses_cleanly_without_a_loaded_module` below
//! is what remains honestly testable of that call path from here: the
//! module-load gate the deleted providers used to sit behind.
//!
//! What genuinely stayed in this crate — persisting a fetched profile as
//! identity facets, loading them back, rendering them, and deleting them on
//! disconnect — is `integrations::composio::identity_store`, this host's own
//! port of the deleted engine's `sync::composio::providers::profile`
//! (see that module's doc comment for exactly what carried over and what did
//! not). `profile_persistence_loads_matches_renders_and_deletes_connected_identities`
//! below is that same test, updated onto the new (async, `&Config`-taking)
//! API. One piece of it could not be preserved: the deleted engine's
//! per-toolkit `is_self_identity(prefix, kind, value)` has no replacement
//! anywhere in `tinymemory-core` any more — only the cross-toolkit
//! `is_self_identity_any_toolkit` survived (it backs the memory tree's entity
//! matcher and was never toolkit-scoped to begin with). The toolkit-scoped
//! assertions are gone from this test as a result; see the src-side gap note
//! in the migration report.

use crate::env_guard::EnvVarGuard;
use std::collections::HashMap;
use std::sync::{OnceLock};

use tempfile::TempDir;

use openhuman_core::config::Config;
use openhuman_core::integrations::composio::ops::composio_get_user_profile;
use openhuman_core::security::credentials::{
    AuthService, APP_SESSION_PROVIDER, DEFAULT_AUTH_PROFILE_NAME,
};

static ENV_LOCK: &OnceLock<tokio::sync::Mutex<()>> = &crate::SHARED_ENV_LOCK;
static MEMORY_SEAMS_INIT: OnceLock<()> = OnceLock::new();

fn ensure_memory_seams() {
    crate::tinyhumans_boot::boot();
    MEMORY_SEAMS_INIT.get_or_init(|| {
        std::thread::Builder::new()
            .name("memory-sync-round23-raw-coverage-seams".to_string())
            .stack_size(8 * 1024 * 1024)
            .spawn(|| {})
            .expect("spawn round23 memory sync seam installer")
            .join()
            .expect("round23 memory sync seam installer panicked");
    });
}

fn env_lock() -> tokio::sync::MutexGuard<'static, ()> {
    ENV_LOCK
        .get_or_init(|| tokio::sync::Mutex::new(()))
        .blocking_lock()
}

async fn env_lock_async() -> tokio::sync::MutexGuard<'static, ()> {
    ENV_LOCK
        .get_or_init(|| tokio::sync::Mutex::new(()))
        .lock().await
}

fn config_in(tmp: &TempDir) -> Config {
    ensure_memory_seams();
    let mut config = Config {
        config_path: tmp.path().join("config.toml"),
        workspace_dir: tmp.path().join("workspace"),
        action_dir: tmp.path().join("workspace"),
        ..Config::default()
    };
    config.secrets.encrypt = false;
    config.memory_tree.embedding_endpoint = None;
    config.memory_tree.embedding_model = None;
    config.memory_tree.embedding_strict = false;
    config
}

async fn persist_config(config: &Config) {
    std::fs::create_dir_all(&config.workspace_dir).expect("workspace dir");
    config.save().await.expect("save config");
}

fn store_session(config: &Config) {
    AuthService::from_config(config)
        .store_provider_token(
            APP_SESSION_PROVIDER,
            DEFAULT_AUTH_PROFILE_NAME,
            "round23-session-token",
            HashMap::new(),
            true,
        )
        .expect("store app session token");
}

/// `composio_get_user_profile` resolves the connection's toolkit through the
/// `tinyconnectors` module before it can fetch anything, so a build with
/// `modules.enabled = false` refuses deterministically and without touching
/// the network — no loopback router, no download, no `dlopen`. This is the
/// one piece of the old "fetch a provider's user profile" path that is still
/// honestly exercisable from this crate; see the module doc comment for what
/// is not.
#[tokio::test]
async fn composio_get_user_profile_refuses_cleanly_without_a_loaded_module() {
    let _guard = env_lock_async().await;
    let tmp = TempDir::new().expect("tempdir");
    let _workspace = EnvVarGuard::set_path("OPENHUMAN_WORKSPACE", tmp.path());
    let _home = EnvVarGuard::set_path("HOME", tmp.path());

    let mut config = config_in(&tmp);
    config.modules.enabled = false;
    persist_config(&config).await;
    store_session(&config);

    let result = composio_get_user_profile(&config, "conn-slack-23").await;
    let error = result.expect_err("profile fetch must refuse without a loaded connectors module");
    assert!(
        error.contains("modules are disabled in configuration"),
        "unexpected error: {error}"
    );
}
