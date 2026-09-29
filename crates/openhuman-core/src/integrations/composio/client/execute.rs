//! [`ComposioClient`]'s tool-discovery surface (`list_tools`). Execution runs
//! in the `tinyconnectors` module; see `execute_dispatch`.

use anyhow::Result;

use super::super::types::ComposioToolsResponse;
use super::connections::ComposioClient;

impl ComposioClient {
    pub async fn list_tools(
        &self,
        toolkits: Option<&[String]>,
        tags: Option<&[String]>,
    ) -> Result<ComposioToolsResponse> {
        let mut params: Vec<String> = Vec::new();
        if let Some(list) = toolkits {
            let joined = list
                .iter()
                .map(|t| t.trim())
                .filter(|t| !t.is_empty())
                .map(|t| urlencoding::encode(t).into_owned())
                .collect::<Vec<_>>()
                .join(",");
            if !joined.is_empty() {
                params.push(format!("toolkits={joined}"));
            }
        }
        if let Some(list) = tags {
            let joined = list
                .iter()
                .map(|t| t.trim())
                .filter(|t| !t.is_empty())
                .map(|t| urlencoding::encode(t).into_owned())
                .collect::<Vec<_>>()
                .join(",");
            if !joined.is_empty() {
                params.push(format!("tags={joined}"));
            }
        }
        let path = if params.is_empty() {
            "/agent-integrations/composio/tools".to_string()
        } else {
            format!("/agent-integrations/composio/tools?{}", params.join("&"))
        };
        tracing::debug!(path = %path, "[composio] list_tools");
        self.inner.get::<ComposioToolsResponse>(&path).await
    }
}
