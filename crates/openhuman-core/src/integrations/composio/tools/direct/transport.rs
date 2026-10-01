//! The host's [`tinyconnectors::client::Transport`] for the direct route.
//!
//! `DirectRoute` owns the paths and the response translation; this owns the
//! wire. It exists (instead of the module's own `ureq` transport) because the
//! host must keep honouring `[proxy]` runtime settings and its TLS roots for
//! Composio traffic, refuse redirects so the `x-api-key` header cannot follow
//! one, and render failures with the exact messages users and the
//! observability classifier already key on.

use std::sync::Arc;

use async_trait::async_trait;
use tinyconnectors::client::{DirectRoute, Transport};
use tinyconnectors::Error as ConnectorError;

use super::http_errors::response_error;
use super::types::DirectComposioClient;

/// Failure label per request path, matching the messages the host has always
/// produced (and the `[composio-direct]` observability anchors match on).
fn failure_label(path: &str) -> &'static str {
    if path.starts_with("/connected_accounts") {
        "Composio v3 connected_accounts failed"
    } else {
        "Composio v3 list_tool_schemas"
    }
}

fn decode_label(path: &str) -> &'static str {
    if path.starts_with("/connected_accounts") {
        "Failed to decode Composio v3 connected_accounts response"
    } else {
        "Failed to decode Composio v3 tools response"
    }
}

fn transport_error(path: &str, message: String) -> ConnectorError {
    ConnectorError::Transport {
        path: path.to_string(),
        message,
    }
}

impl DirectComposioClient {
    /// The module's direct route over this client's transport.
    ///
    /// Built per call: the host's process-wide key gate (`direct_auth`) is what
    /// persists across calls, so the route's own per-instance gate never trips
    /// here by design.
    pub(crate) fn route(self: &Arc<Self>) -> DirectRoute {
        DirectRoute::new(self.clone(), &self.api_key, "default")
    }
}

/// Collapse a route failure into the message the transport produced.
///
/// The route wraps transport failures as `request to <path> failed: <msg>`;
/// users have only ever seen `<msg>`.
pub(crate) fn route_error(error: ConnectorError) -> anyhow::Error {
    match error {
        ConnectorError::Transport { message, .. } => anyhow::anyhow!(message),
        other => anyhow::anyhow!(other.to_string()),
    }
}

#[async_trait]
impl Transport for DirectComposioClient {
    async fn get(&self, path: &str) -> tinyconnectors::Result<serde_json::Value> {
        let url = format!("{}{}", self.base_v3, path);
        self.ensure_request_url(&url)
            .map_err(|e| transport_error(path, format!("{e:#}")))?;
        tracing::debug!(path = %path.split('?').next().unwrap_or(path), "[composio-direct] transport GET");

        let resp = self
            .client()
            .get(&url)
            .header("x-api-key", &self.api_key)
            .send()
            .await
            .map_err(|e| transport_error(path, format!("{:#}", anyhow::Error::from(e))))?;

        if !resp.status().is_success() {
            let err = response_error(resp).await;
            return Err(transport_error(
                path,
                format!("{}: {err}", failure_label(path)),
            ));
        }

        resp.json::<serde_json::Value>().await.map_err(|e| {
            transport_error(path, format!("{}: {e}", decode_label(path)))
        })
    }

    async fn post(
        &self,
        path: &str,
        _body: &serde_json::Value,
    ) -> tinyconnectors::Result<serde_json::Value> {
        Err(transport_error(
            path,
            "the host direct transport serves reads only; writes run in the connector module"
                .to_string(),
        ))
    }

    async fn delete(&self, path: &str) -> tinyconnectors::Result<serde_json::Value> {
        Err(transport_error(
            path,
            "the host direct transport serves reads only; writes run in the connector module"
                .to_string(),
        ))
    }
}
