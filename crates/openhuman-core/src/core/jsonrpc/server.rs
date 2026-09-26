//! Server entry points kept for the hosts that predate
//! [`CoreBuilder`](crate::core::runtime::CoreBuilder).
//!
//! Each `run_server*` function is a thin shim that composes a `CoreBuilder`
//! and calls [`CoreRuntime::serve`](crate::core::runtime::CoreRuntime::serve).

use tokio_util::sync::CancellationToken;

/// Resolves the port for the core server from environment variables or defaults.
#[cfg(feature = "http-server")]
pub(crate) fn core_port() -> u16 {
    std::env::var("OPENHUMAN_CORE_PORT")
        .ok()
        .and_then(|v| v.parse::<u16>().ok())
        .unwrap_or(7788)
}

/// Resolves the bind address host for the core server from environment variables or defaults.
#[cfg(feature = "http-server")]
pub(crate) fn core_host() -> String {
    std::env::var("OPENHUMAN_CORE_HOST")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "127.0.0.1".to_string())
}

/// Metadata sent back to the Tauri host once the embedded core has selected
/// and bound its listen port.
#[derive(Debug, Clone)]
pub struct EmbeddedReadySignal {
    pub port: u16,
    pub fallback_from: Option<u16>,
}

/// Runs the HTTP/JSON-RPC server.
///
/// This function binds to the specified host and port, initializes the router,
/// bootstraps long-lived runtime infrastructure, and starts serving requests.
pub async fn run_server(
    host: Option<&str>,
    port: Option<u16>,
    socketio_enabled: bool,
) -> anyhow::Result<()> {
    run_server_inner(host, port, socketio_enabled, false, None, None, None).await
}

/// Runs the request/response-only HTTP API without detached background jobs.
pub async fn run_server_headless(host: Option<&str>, port: Option<u16>) -> anyhow::Result<()> {
    let services = crate::core::runtime::ServiceSet::headless_api();
    run_server_with_services(host, port, services, false, None, None, None).await
}

/// Like [`run_server`] but marks the instance as embedded.
pub async fn run_server_embedded(
    host: Option<&str>,
    port: Option<u16>,
    socketio_enabled: bool,
    shutdown_token: CancellationToken,
) -> anyhow::Result<()> {
    run_server_inner(
        host,
        port,
        socketio_enabled,
        true,
        Some(shutdown_token),
        None,
        None,
    )
    .await
}

/// Embedded entrypoint with an explicit readiness callback.
///
/// When the caller already holds the per-launch RPC bearer in memory (the
/// Tauri shell now that the core runs in-process — PR #1061), it should
/// pass `Some(token)` so the embedded server can seed its auth subsystem
/// via [`crate::core::auth::init_rpc_token_with_value`] without ever
/// reading `OPENHUMAN_CORE_TOKEN` from the process environment.  Passing
/// `None` preserves the env-as-config fallback (CLI / docker / cloud).
pub async fn run_server_embedded_with_ready(
    host: Option<&str>,
    port: Option<u16>,
    socketio_enabled: bool,
    shutdown_token: CancellationToken,
    ready_tx: tokio::sync::oneshot::Sender<EmbeddedReadySignal>,
    rpc_token: Option<std::sync::Arc<String>>,
) -> anyhow::Result<()> {
    run_server_inner(
        host,
        port,
        socketio_enabled,
        true,
        Some(shutdown_token),
        Some(ready_tx),
        rpc_token,
    )
    .await
}

/// Internal server entrypoint.
async fn run_server_inner(
    host: Option<&str>,
    port: Option<u16>,
    socketio_enabled: bool,
    embedded_core: bool,
    shutdown_token: Option<CancellationToken>,
    ready_tx: Option<tokio::sync::oneshot::Sender<EmbeddedReadySignal>>,
    rpc_token: Option<std::sync::Arc<String>>,
) -> anyhow::Result<()> {
    let mut services = crate::core::runtime::ServiceSet::desktop();
    services.socketio = socketio_enabled;
    run_server_with_services(
        host,
        port,
        services,
        embedded_core,
        shutdown_token,
        ready_tx,
        rpc_token,
    )
    .await
}

async fn run_server_with_services(
    host: Option<&str>,
    port: Option<u16>,
    services: crate::core::runtime::ServiceSet,
    embedded_core: bool,
    shutdown_token: Option<CancellationToken>,
    ready_tx: Option<tokio::sync::oneshot::Sender<EmbeddedReadySignal>>,
    rpc_token: Option<std::sync::Arc<String>>,
) -> anyhow::Result<()> {
    // `run_server_inner` is now a thin shim over the CoreBuilder/CoreRuntime
    // composition (Phase 1). It reproduces the legacy behavior exactly: all
    // background services on (`ServiceSet::desktop`), Socket.IO per the caller
    // flag, and the legacy `embedded_core` → `HostKind` mapping (embedded ==
    // Tauri shell; standalone splits CLI / Docker via `detect_standalone`).
    // See the pluggable-core work (`core::runtime`).
    let host_kind = if embedded_core {
        crate::core::types::HostKind::TauriShell
    } else {
        crate::core::types::HostKind::detect_standalone()
    };
    let token = match rpc_token {
        Some(token) => crate::core::runtime::TokenSource::Fixed(token),
        None => crate::core::runtime::TokenSource::EnvOrFile,
    };
    let mut builder = crate::core::runtime::CoreBuilder::new(host_kind)
        .token(token)
        .services(services);
    if let Some(host) = host {
        builder = builder.host(host);
    }
    if let Some(port) = port {
        builder = builder.port(port);
    }

    let runtime = builder.build().await?;
    runtime.serve(ready_tx, shutdown_token).await
}

#[cfg(test)]
#[path = "server_tests.rs"]
mod tests;
