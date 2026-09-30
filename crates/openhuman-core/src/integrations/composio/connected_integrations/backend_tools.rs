//! Backend tool-schema read used while the connector module is on the direct
//! route.
//!
//! In direct mode the connector module holds exactly one route — the user's own
//! key — so it cannot also ask the TinyHumans backend for its curated tool
//! catalogue. Schemas are tenant-agnostic, and the prompt overview wants the
//! curated ones when a backend session happens to exist, so this one
//! authenticated GET stays on the host until the module can answer a member on
//! a per-call route (pending a module member and release).

use anyhow::Result;

use super::super::types::ComposioToolsResponse;
use crate::config::Config;

/// Fetch the backend's curated schemas for `toolkits`.
///
/// `None` when there is no backend credential (the caller falls back to lazy
/// resolution at delegation time); `Some(Err)` when the call itself failed.
pub(super) async fn fetch_backend_tool_schemas(
    config: &Config,
    toolkits: &[String],
) -> Option<Result<ComposioToolsResponse>> {
    let client = crate::integrations::build_client(config)?;
    let joined = toolkits
        .iter()
        .map(|t| t.trim())
        .filter(|t| !t.is_empty())
        .map(|t| urlencoding::encode(t).into_owned())
        .collect::<Vec<_>>()
        .join(",");
    let path = if joined.is_empty() {
        "/agent-integrations/composio/tools".to_string()
    } else {
        format!("/agent-integrations/composio/tools?toolkits={joined}")
    };
    tracing::debug!(path = %path, "[composio] backend tool schemas (direct-mode overview)");
    Some(client.get::<ComposioToolsResponse>(&path).await)
}
