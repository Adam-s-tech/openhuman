//! Boilerplate shared by the memory RPC suites that serve the core router
//! in-process (`memory_sources_e2e`, `memory_tree_health_e2e`,
//! `tree_summarizer_e2e`): a minimal workspace config, a served router, and the
//! `Outcome`-unwrapping `ok`.
//!
//! Declare it at the root of an aggregated target next to `rpc_auth`.

#![allow(dead_code)]

use openhuman_rpc::server::build_core_http_router;
use serde_json::Value;
use std::path::Path;

use crate::rpc_auth::ensure_rpc_auth;

/// Install the memory module's policy once per process, on a thread with a
/// stack deep enough for the policy build (a default test stack overflows).
pub fn ensure_memory_seams() {
    // Kept as a no-op for call-site compatibility; memory is hosted directly
    // by the core and no longer requires a loadable memory module.
}

/// Write the minimal workspace config (plus the `users/local` copy) under `dir`.
pub fn write_config(dir: &Path) {
    std::fs::create_dir_all(dir).expect("mkdir");
    let cfg = r#"
default_model = "e2e-mock-model"
default_temperature = 0.7

[secrets]
encrypt = false

[memory_tree]
embedding_strict = false
"#;
    std::fs::write(dir.join("config.toml"), cfg).expect("write config");

    let user_dir = dir.join("users").join("local");
    std::fs::create_dir_all(&user_dir).expect("mkdir user dir");
    std::fs::write(user_dir.join("config.toml"), cfg).expect("write user config");
}

/// Serve the core router; returns the base URL.
pub async fn serve() -> (String, tokio::task::JoinHandle<Result<(), std::io::Error>>) {
    ensure_memory_seams();
    ensure_rpc_auth();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().expect("addr");
    let handle =
        tokio::spawn(async move { axum::serve(listener, build_core_http_router(false)).await });
    (format!("http://{addr}"), handle)
}

/// The envelope's payload: `result.result` when the `Outcome` wraps one next to
/// its `logs`, else `result`. Panics with `ctx` on a JSON-RPC error.
pub fn ok(v: &Value, ctx: &str) -> Value {
    if let Some(err) = v.get("error") {
        panic!("{ctx}: JSON-RPC error: {err}");
    }
    let outer = v
        .get("result")
        .unwrap_or_else(|| panic!("{ctx}: missing result: {v}"));
    if let Some(inner) = outer.get("result") {
        inner.clone()
    } else {
        outer.clone()
    }
}
