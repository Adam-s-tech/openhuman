//! Server-Sent Event streams: `/events`, `/events/webhooks`, `/events/domain`.

use axum::extract::Query;
use axum::http::{header, StatusCode};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use tokio_stream::StreamExt;

/// Query parameters for the events SSE endpoint.
///
/// `client_id` selects which broadcast events to forward; `token` is the
/// single-shot bind token minted by the `core.events_subscribe_token` RPC.
/// Both are required — browser `EventSource` cannot attach an
/// `Authorization` header, so the bind token is the only credential the
/// endpoint accepts.
#[derive(Debug, serde::Deserialize)]
pub(super) struct EventsQuery {
    client_id: String,
    #[serde(default)]
    token: Option<String>,
}

/// Handler for the main events SSE endpoint.
///
/// Accepts either of two credentials:
/// 1. `Authorization: Bearer <core token>` — used by CLI tooling, the
///    Tauri shell via `core_rpc_relay`, and the in-tree e2e suite that
///    can set HTTP headers directly. Validated against the same
///    per-process bearer the rest of `/rpc` uses.
/// 2. `?token=<bind>` minted via the `core.events_subscribe_token` RPC
///    — used by browser `EventSource`, which cannot attach custom
///    headers. The token is bound to a specific `client_id` and is
///    consumed on validation so a leaked URL cannot be replayed.
///
/// Both paths converge on the same broadcast stream filtered by
/// `client_id`.
pub(super) async fn events_handler(
    headers: axum::http::HeaderMap,
    Query(query): Query<EventsQuery>,
) -> Response {
    let bearer = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let bearer_ok = bearer
        .map(openhuman_core::core::auth::verify_bearer_token)
        .unwrap_or(false);

    if !bearer_ok {
        let supplied_token = query
            .token
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty());
        let Some(supplied_token) = supplied_token else {
            log::warn!(
                "[events] reject subscribe: missing bind token + missing bearer (client_id_len={})",
                query.client_id.len()
            );
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "ok": false,
                    "error": "unauthorized",
                    "message": "Missing credentials. Supply 'Authorization: Bearer <core>' or mint a bind token with the `core.events_subscribe_token` RPC and pass it as ?token="
                })),
            )
                .into_response();
        };
        if !openhuman_core::core::event_bind_tokens::consume(&query.client_id, supplied_token) {
            log::warn!(
                "[events] reject subscribe: bind token invalid or expired (client_id_len={})",
                query.client_id.len()
            );
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "ok": false,
                    "error": "unauthorized",
                    "message": "Bind token is unknown, expired, or bound to a different client_id."
                })),
            )
                .into_response();
        }
    }

    let client_id = query.client_id;
    let rx = openhuman_core::web_chat::subscribe_web_channel_events();
    let stream = tokio_stream::wrappers::BroadcastStream::new(rx).filter_map(
        move |item| -> Option<Result<Event, std::convert::Infallible>> {
            let event = match item {
                Ok(ev) => ev,
                Err(_) => return None,
            };
            if event.client_id != client_id {
                return None;
            }
            let data = match serde_json::to_string(&event) {
                Ok(data) => data,
                Err(_) => return None,
            };
            Some(Ok(Event::default().event(event.event).data(data)))
        },
    );

    Sse::new(stream)
        .keep_alive(KeepAlive::new().interval(std::time::Duration::from_secs(10)))
        .into_response()
}

/// Handler for the webhook debug events SSE endpoint.
pub(super) async fn webhook_events_handler() -> Response {
    let stream = tokio_stream::once(Ok::<Event, std::convert::Infallible>(
        Event::default()
            .event("webhooks_debug")
            .data("{\"event_type\":\"runtime_removed\"}"),
    ));
    Sse::new(stream)
        .keep_alive(KeepAlive::new().interval(std::time::Duration::from_secs(10)))
        .into_response()
}

/// SSE endpoint streaming DomainEvent bus events for the live event log panel.
///
/// Requires bearer auth. Streams all domain events as JSON with event type
/// set to the domain name (agent, tool, memory, etc.).
pub(super) async fn domain_events_handler(headers: axum::http::HeaderMap) -> Response {
    let bearer = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let bearer_ok = bearer
        .map(openhuman_core::core::auth::verify_bearer_token)
        .unwrap_or(false);

    if !bearer_ok {
        log::warn!("[events/domain] reject subscribe: missing or invalid bearer token");
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "ok": false,
                "error": "unauthorized",
                "message": "Bearer token required for domain event stream"
            })),
        )
            .into_response();
    }

    // Read dashboard config for event stream settings.
    let es_cfg = openhuman_core::config::rpc::load_config_with_timeout()
        .await
        .map(|c| c.dashboard.event_stream)
        .unwrap_or_default();

    if !es_cfg.enabled {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "ok": false, "error": "event stream disabled by config" })),
        )
            .into_response();
    }

    let bus = match openhuman_core::core::bus::BUS.get() {
        Some(bus) => bus,
        None => {
            log::warn!("[events/domain] event bus not initialized");
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({ "ok": false, "error": "event bus not initialized" })),
            )
                .into_response();
        }
    };

    log::debug!("[events/domain] client connected, streaming domain events");

    // The active workspace, resolved once here so a client that connects
    // mid-life starts out knowing which rows are its own rather than
    // waiting for the next event to tell it (#5966). This is the one place
    // in this handler that can afford the authoritative read — it happens
    // per connection, not per event — and it refills the cache the row
    // stamping below relies on.
    let active_workspace = openhuman_core::config::active_workspace_dir()
        .await
        .map(|dir| openhuman_core::config::workspace_handle(&dir))
        .map_err(|error| {
            log::warn!(
                "[events/domain] could not resolve the active workspace ({error}); \
                 the client will scope the log once an event says which workspace is active"
            );
        })
        .ok();

    // Send config as first SSE event so frontend can apply settings.
    let config_event = Event::default().event("config").data(
        serde_json::to_string(&json!({
            "max_entries": es_cfg.max_entries,
            "new_entries": es_cfg.new_entries,
            "active_workspace": active_workspace,
        }))
        .unwrap_or_default(),
    );

    // `BroadcastStream` wraps a raw `broadcast::Receiver`; tinybus hands back a
    // decoding receiver instead, so the stream is built by unfolding it. Lag is
    // already handled inside `recv`, which is why there is no error arm to
    // filter out any more.
    let event_stream = futures::stream::unfold(bus.receiver(), |mut rx| async move {
        rx.recv()
            .await
            .map(|event| (Ok::<_, std::convert::Infallible>(event), rx))
    })
    .filter_map(|item| -> Option<Result<Event, std::convert::Infallible>> {
        let event = match item {
            Ok(ev) => ev,
            Err(_) => return None,
        };
        let domain = event.domain().to_string();
        let event_name = event.variant_name();
        let agent = event.agent_hint().unwrap_or("").to_string();
        // Most variants say everything in their name; the ones whose point is
        // a failure *reason* would otherwise reach the log with the reason
        // discarded, so they opt into one already-redacted line (#5931). It is
        // `null` for every other variant, which renders as no change.
        let detail = event.log_detail();
        // Which workspace this row belongs to, and which one is current
        // (#5966). One process serves more than one workspace over its life,
        // so without these two a row left over from a workspace the user has
        // switched away from is indistinguishable from one belonging to the
        // workspace they are in.
        //
        // Both are *handles*, never `workspace_dir` itself: this envelope
        // feeds a settings panel and its NDJSON download, and the path is
        // under the user's home directory.
        //
        // `active` is read from the cache rather than resolved. This closure
        // is synchronous — `tokio_stream`'s `filter_map` — so it could not
        // await a resolve, and it runs for every domain event the process
        // publishes, so it should not want to. `None` means "not resolved
        // since the last workspace marker write", which the client treats as
        // unknown rather than as a mismatch.
        let workspace = event
            .workspace_dir()
            .map(openhuman_core::config::workspace_handle);
        let active = openhuman_core::config::active_workspace_dir_cached()
            .map(|dir| openhuman_core::config::workspace_handle(&dir));
        let data = json!({
            "domain": domain,
            "event": event_name,
            "agent": agent,
            "detail": detail,
            "workspace": workspace,
            "active_workspace": active,
            "timestamp": chrono::Utc::now().format("%H:%M:%S").to_string(),
        });
        let data_str = serde_json::to_string(&data).ok()?;
        Some(Ok(Event::default().event(domain).data(data_str)))
    });

    let config_stream =
        futures::stream::once(async move { Ok::<_, std::convert::Infallible>(config_event) });
    let stream = config_stream.chain(event_stream);

    Sse::new(stream)
        .keep_alive(KeepAlive::new().interval(std::time::Duration::from_secs(5)))
        .into_response()
}
