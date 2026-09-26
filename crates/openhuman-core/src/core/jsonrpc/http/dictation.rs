//! `GET /ws/dictation`: the authenticated voice-dictation WebSocket.

use std::sync::Arc;

use axum::extract::{Query, WebSocketUpgrade};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

/// Query parameters for the dictation WebSocket endpoint.
///
/// Browser `WebSocket` cannot attach an `Authorization` header on upgrade, so
/// the FE forwards the per-process core bearer as a `?token=…` query param —
/// validated against the same in-process RPC token via [`verify_bearer_token`]
/// (single source of truth, no separate credential).
#[derive(Debug, serde::Deserialize)]
struct DictationQuery {
    #[serde(default)]
    token: Option<String>,
}

/// WebSocket upgrade handler for streaming voice dictation.
///
/// Authenticated before upgrade (C4 / issue #1924): the request must carry the
/// per-process core bearer either as `Authorization: Bearer <token>` (CLI /
/// native callers) or as `?token=<token>` (browser `WebSocket`, which cannot
/// set headers), and — when an `Origin` header is present — that origin must be
/// on the local-app allowlist, mirroring the Socket.IO handshake check. Missing
/// or wrong credentials are rejected with 401 and the socket is never upgraded.
pub(super) async fn dictation_ws_handler(
    headers: axum::http::HeaderMap,
    Query(query): Query<DictationQuery>,
    ws: WebSocketUpgrade,
) -> Response {
    log::info!("[ws] dictation WebSocket upgrade requested");

    // Origin check (same allowlist Socket.IO enforces): native clients send no
    // Origin and are accepted; cross-origin browser pages are rejected even if
    // they somehow hold the bearer.
    let origin = headers
        .get(header::ORIGIN)
        .and_then(|v| v.to_str().ok())
        .map(str::trim);
    if !crate::core::socketio::origin_is_allowed(origin) {
        log::warn!("[ws] dictation upgrade rejected: disallowed origin {origin:?}");
        return (
            StatusCode::FORBIDDEN,
            Json(json!({
                "ok": false,
                "error": "forbidden",
                "message": "Origin not allowed for the dictation WebSocket."
            })),
        )
            .into_response();
    }

    // Bearer check: header first, then `?token=` for browser WebSocket clients.
    let header_token = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let bearer_ok = header_token
        .map(crate::core::auth::verify_bearer_token)
        .unwrap_or(false);
    let bearer_ok = bearer_ok
        || query
            .token
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(crate::core::auth::verify_bearer_token)
            .unwrap_or(false);
    if !bearer_ok {
        log::warn!("[ws] dictation upgrade rejected: missing or invalid bearer token");
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "ok": false,
                "error": "unauthorized",
                "message": "Missing or invalid token. Supply 'Authorization: Bearer <core>' or ?token=<core>."
            })),
        )
            .into_response();
    }

    ws.on_upgrade(|socket| async move {
        let config = match crate::config::rpc::load_config_with_timeout().await {
            Ok(c) => Arc::new(c),
            Err(e) => {
                log::error!("[ws] failed to load config for dictation: {e}");
                return;
            }
        };
        crate::voice::streaming::handle_dictation_ws(socket, config).await;
    })
}
