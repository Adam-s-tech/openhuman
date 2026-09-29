//! Searchable agent tools for actions on already-connected MCP servers.
//!
//! What each action tool *is* — its provider-safe name, description, family
//! and sanitized schema, and the server-then-tool ordering — is the
//! contract's (`tinymcp_bus::agent_tools::action_tool_specs`), so a tool's
//! identity is derived in one place. What stays here is host policy: which
//! remote definitions pass this application's injection scan
//! ([`super::tools_safe_for_agent`], applied when listing and re-checked when
//! calling), the permission and exposure a spec maps to, and the call itself.

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;
use tinytools::{PermissionLevel, Tool, ToolCategory, ToolExposure, ToolResult};

use crate::config::Config;

pub use tinymcp_bus::agent_tools::searchable_name;
use tinymcp_bus::agent_tools::{action_tool_specs, ActionToolSpec};

use super::connections;
use super::tools::{exposure_for, permission_for};
use super::types::ConnectedServerOverview;

/// One deferred registration per callable action from each connected server.
/// Remote definitions are filtered by the host's injection policy first.
pub fn deferred_connected_tools(
    config: Arc<Config>,
    servers: &[ConnectedServerOverview],
) -> Vec<Box<dyn Tool>> {
    action_tool_specs(servers, super::tools_safe_for_agent)
        .into_iter()
        .map(|action| Box::new(McpActionTool::new(Arc::clone(&config), action)) as Box<dyn Tool>)
        .collect()
}

struct McpActionTool {
    config: Arc<Config>,
    action: ActionToolSpec,
}

impl McpActionTool {
    fn new(config: Arc<Config>, action: ActionToolSpec) -> Self {
        Self { config, action }
    }
}

#[async_trait]
impl Tool for McpActionTool {
    fn name(&self) -> &str {
        &self.action.spec.name
    }

    fn description(&self) -> &str {
        &self.action.spec.description
    }

    fn parameters_schema(&self) -> Value {
        self.action.spec.parameters.clone()
    }

    fn permission_level(&self) -> PermissionLevel {
        permission_for(self.action.spec.effect)
    }

    fn external_effect(&self) -> bool {
        true
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::Workflow
    }

    fn exposure(&self) -> ToolExposure {
        exposure_for(&self.action.spec)
    }

    fn family(&self) -> Option<&str> {
        Some(&self.action.family)
    }

    async fn execute(&self, arguments: Value) -> anyhow::Result<ToolResult> {
        let server_id = &self.action.server_id;
        let tool_name = &self.action.tool_name;
        let Some(live_tools) = connections::server_tools_for_config(&self.config, server_id).await
        else {
            return Ok(ToolResult::error("MCP server is no longer connected"));
        };
        // Re-checked at call time: the server may have changed a definition
        // since the tool was listed.
        let safe_tools = super::tools_safe_for_agent(server_id, live_tools);
        if !safe_tools.iter().any(|tool| &tool.name == tool_name) {
            return Ok(ToolResult::error("MCP tool is no longer available"));
        }
        let outcome = super::ops::mcp_clients_tool_call(
            &self.config,
            server_id.clone(),
            tool_name.clone(),
            arguments,
        )
        .await
        .map_err(anyhow::Error::msg)?;
        let payload = serde_json::to_string(&outcome.value)?;
        if outcome.value.get("is_error").and_then(Value::as_bool) == Some(true) {
            Ok(ToolResult::error(payload))
        } else {
            Ok(ToolResult::success(payload))
        }
    }
}

#[cfg(test)]
#[path = "action_tool_tests.rs"]
mod tests;
