//! Direct-mode response reshapers: `direct_list_connections`, and `direct_list_tools`.
//! Call Composio's v3 API directly (via a bound [`crate::tools::DirectComposioClient`])
//! and reshape the v3 response into the canonical envelope types, so downstream
//! callers don't have to branch on mode.

use std::sync::Arc;

use super::super::direct_auth;
use super::super::types::{ComposioConnectionsResponse, ComposioToolsResponse};

/// Direct-mode connection listing.
///
/// Calls Composio v3 `/connected_accounts` (via
/// [`crate::tools::DirectComposioClient::list_connected_accounts`])
/// and maps each item to the canonical [`ComposioConnection`] so the
/// existing frontend type contract and the 5 s UI poll keep working
/// unchanged.
///
/// Toolkit slug, status, and `created_at` are extracted defensively —
/// missing or unparseable fields fall back to empty strings / `None`
/// rather than dropping the row. The status filter applied downstream
/// (`ComposioConnection::is_active`) treats empty status as inactive,
/// so a malformed row will simply not be presented as connected — the
/// fail-safe shape the user expects.
pub async fn direct_list_connections(
    direct: &Arc<crate::tools::DirectComposioClient>,
) -> anyhow::Result<ComposioConnectionsResponse> {
    tracing::debug!("[composio-direct] list_connections: GET v3 /connected_accounts");
    let key_id = direct.auth_key_fingerprint();
    if let Some(error) = direct_auth::direct_auth_backoff_error(key_id) {
        tracing::warn!(
            "[composio-direct] list_connections: direct API key backoff gate open; \
             skipping v3 /connected_accounts"
        );
        anyhow::bail!("{error}");
    }

    let response = match direct.list_connections().await {
        Ok(response) => {
            direct_auth::record_direct_auth_success(key_id);
            response
        }
        Err(error) => {
            let rendered = format!("{error:#}");
            match direct_auth::record_direct_auth_failure(key_id, &rendered) {
                direct_auth::DirectAuthFailureDecision::NotAuthFailure => {}
                direct_auth::DirectAuthFailureDecision::RetryAllowed { consecutive } => {
                    tracing::warn!(
                        consecutive,
                        threshold = direct_auth::DIRECT_INVALID_API_KEY_THRESHOLD,
                        "[composio-direct] list_connections: direct API key rejected"
                    );
                }
                direct_auth::DirectAuthFailureDecision::CircuitOpened { consecutive } => {
                    let backoff = direct_auth::invalid_api_key_backoff_message(consecutive);
                    tracing::warn!(
                        consecutive,
                        threshold = direct_auth::DIRECT_INVALID_API_KEY_THRESHOLD,
                        "[composio-direct] list_connections: direct API key backoff gate opened"
                    );
                    anyhow::bail!("{backoff}");
                }
            }
            return Err(error);
        }
    };
    tracing::debug!(
        count = response.connections.len(),
        "[composio-direct] list_connections: mapped v3 connected accounts"
    );
    Ok(response)
}

/// Direct-mode tool listing. Calls
/// Composio v3 `/tools?toolkits=<csv>&tags=<a>&tags=<b>` via
/// [`crate::tools::DirectComposioClient::list_tool_schemas_v3`] and
/// reshapes each item into the same [`ComposioToolSchema`] envelope the
/// backend-proxied path returns.
///
/// `toolkits` may be empty (full direct-tenant catalogue) or scoped to
/// the user's connected toolkits (preferred — keeps response size bounded
/// and skips schemas the agent can't actually call). `composio_list_tools`'s
/// direct branch passes `direct_list_connections`'s active set.
///
/// `tags` mirrors the backend path's tag filter so a self-key user's
/// `composio_list_tools(..., tags)` request narrows by Composio action tag
/// in direct mode too (previously the tag filter was silently dropped on
/// the direct branch). The caller is expected to have already applied
/// [`crate::integrations::composio::ops::should_forward_tags`] before passing `tags` here.
///
/// Schemas surfaced here are tenant-agnostic — Composio's action
/// definitions are the same across tenants, so direct-mode users get
/// the same model-callable shape backend-mode does. Downstream curated-
/// whitelist filtering (`evaluate_tool_visibility` / `find_curated`)
/// still applies at the `ops::composio_list_tools` layer.
///
/// `pub(crate)` (widened from `pub(super)`) so
/// `catalog::fetch_raw_toolkit_tools` can call this directly for
/// the LIVE (uncurated) tool-contract catalog the Workflow builder grounds
/// against — that caller deliberately bypasses `composio_list_tools`'s
/// curated-whitelist filter (`filter_list_tools_response_for_direct`),
/// which this function never applies itself; the filter is layered on by
/// its `composio_list_tools` caller, not baked in here.
pub(crate) async fn direct_list_tools(
    direct: &Arc<crate::tools::DirectComposioClient>,
    toolkits: &[String],
    tags: Option<&[String]>,
) -> anyhow::Result<ComposioToolsResponse> {
    tracing::debug!(
        toolkits = toolkits.len(),
        tags = tags.map(<[String]>::len).unwrap_or(0),
        "[composio-direct] list_tools: GET v3 /tools"
    );
    let response = direct.list_tools(toolkits, tags.unwrap_or(&[])).await?;
    tracing::debug!(
        count = response.tools.len(),
        "[composio-direct] list_tools: mapped v3 tool schemas"
    );
    Ok(response)
}
