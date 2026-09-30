//! Connected-account listing for the direct route.

use super::http_errors::response_error;
use super::types::DirectComposioClient;
use anyhow::Context;
use serde::Deserialize;

impl DirectComposioClient {
    /// List the user's connected accounts on Composio v3.
    ///
    /// GET `https://backend.composio.dev/api/v3/connected_accounts` with
    /// `x-api-key: <user_key>`. Returns the raw item list; reshaping
    /// into [`super::super::super::composio::types::ComposioConnection`]
    /// happens at the call site in `composio/client.rs::direct_list_connections`.
    ///
    /// The v3 envelope is `{ items: [{ id, status, toolkit, created_at, ... }] }`.
    /// Toolkit may arrive as either a plain string slug or as a nested
    /// object — we tolerate both via [`ComposioConnectedAccount::toolkit_slug`].
    /// This matches the same upstream shape drift handled by
    /// `de_string_or_object` in `composio/types.rs`.
    pub async fn list_connected_accounts(&self) -> anyhow::Result<Vec<ComposioConnectedAccount>> {
        let url = format!("{}/connected_accounts", self.base_v3);
        self.ensure_request_url(&url)?;

        let resp = self
            .client()
            .get(&url)
            .header("x-api-key", &self.api_key)
            // Composio paginates; pull a generous page size so most
            // users see their full list in one round trip. If a user has
            // > 200 connected accounts (extremely rare for an individual
            // tenant) the rest will be missing until we add explicit
            // pagination — note for the follow-up.
            .query(&[("limit", "200")])
            .send()
            .await?;

        if !resp.status().is_success() {
            let err = response_error(resp).await;
            anyhow::bail!("Composio v3 connected_accounts failed: {err}");
        }

        let mut body: ComposioConnectedAccountsResponse = resp
            .json()
            .await
            .context("Failed to decode Composio v3 connected_accounts response")?;
        // Drop rows with a blank id — serde_default means id can be ""
        // if the upstream response is malformed. An empty connectionId
        // propagated downstream causes invalid v3 API calls.
        body.items.retain(|item| !item.id.trim().is_empty());
        tracing::debug!(
            count = body.items.len(),
            "[composio-direct] list_connected_accounts: fetched connected accounts"
        );
        Ok(body.items)
    }
}

#[derive(Debug, Deserialize)]
struct ComposioConnectedAccountsResponse {
    #[serde(default)]
    items: Vec<ComposioConnectedAccount>,
}

/// One v3 connected-account row.
///
/// Field shapes follow Composio's v3 docs as of May 2026. `toolkit` may
/// be either a string slug (older payloads) or a nested object with a
/// `slug` field (newer payloads); [`Self::toolkit_slug`] extracts the
/// canonical slug from either shape.
#[derive(Debug, Clone, Deserialize)]
pub struct ComposioConnectedAccount {
    #[serde(default)]
    pub id: String,
    /// `"ACTIVE"`, `"INITIATED"`, `"FAILED"`, … — passed through as-is
    /// so the caller's status filter (`ComposioConnection::is_active`)
    /// applies uniformly across both backend-proxied and direct paths.
    #[serde(default)]
    pub status: Option<String>,
    /// Composio uses `created_at` (snake_case) at v3. We keep both
    /// spellings to tolerate any upstream drift back to `createdAt`.
    #[serde(default, alias = "createdAt")]
    pub created_at: Option<String>,
    /// Toolkit may be a plain string slug or a nested
    /// `ComposioToolkitRef`. Extracted via [`Self::toolkit_slug`].
    #[serde(default)]
    pub(super) toolkit: Option<serde_json::Value>,
    /// Older payload shape — a top-level `app_name` string. Used as
    /// a fallback when `toolkit` is absent or unparseable.
    #[serde(default, rename = "appName", alias = "app_name")]
    pub(super) app_name: Option<String>,
}

impl ComposioConnectedAccount {
    /// Best-effort extract of the toolkit slug from the
    /// possibly-polymorphic `toolkit` field, falling back to
    /// `app_name`. Returns `None` only when no recognizable slug
    /// representation is present.
    pub fn toolkit_slug(&self) -> Option<String> {
        if let Some(value) = &self.toolkit {
            match value {
                serde_json::Value::String(s) => {
                    let t = s.trim();
                    if !t.is_empty() {
                        return Some(t.to_string());
                    }
                }
                serde_json::Value::Object(map) => {
                    for key in ["slug", "id", "name", "key"] {
                        if let Some(serde_json::Value::String(s)) = map.get(key) {
                            let t = s.trim();
                            if !t.is_empty() {
                                return Some(t.to_string());
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        self.app_name
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
    }
}
