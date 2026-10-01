//! [`ToolPolicyMiddleware`]: enforce the builder-configured `ToolPolicy` and
//! the session's channel-permission ceiling at the tool boundary, and render
//! session-scoped `use_skill` listings.

use std::sync::Arc;

use async_trait::async_trait;

use tinyagents_harness::context::RunContext;
use tinyagents_harness::error::Result as TaResult;
use tinyagents_harness::middleware::{
    MiddlewareToolOutcome, PolicyDecision, ToolCallPolicy, ToolHandler, ToolMiddleware,
    ToolPolicyGate,
};
use tinyinference_llm::tool::ToolCall as TaToolCall;
use tinytools::ToolResult as TaToolResult;

use crate::agent::tinyagents::host::OpenHumanRunContext;
use crate::agent::tinyagents::policy_denial::PolicyDenial;
use tinytools::Tool;

/// `wrap_tool`: enforce the agent's builder-configured [`ToolPolicy`] at the tool
/// boundary (issue #4249). The in-house engine ran this check in
/// `agent_tool_exec` (`ctx.tool_policy.check(...)`); the tinyagents path bypassed
/// it, so a `.tool_policy()` deny/require-approval silently no-opped and the tool
/// executed anyway — a security regression. This middleware restores it: a
/// blocking decision short-circuits with a model-consumable result carrying the
/// same `"Tool '<name>' <denied|requires approval> by policy '<policy>': <reason>"`
/// wording the engine produced.
pub(crate) struct ToolPolicyMiddleware {
    /// The builder policy behind the harness gate, adapted to the harness
    /// `ToolCallPolicy` seam (see [`SessionToolPolicy`]).
    gate: ToolPolicyGate<OpenHumanRunContext>,
    /// The session's channel-permission snapshot — enforces the per-channel deny
    /// + per-call permission-level ceiling the engine ran in `agent_tool_exec`.
    session: crate::tools::agent_policy::ToolPolicySession,
    /// Shared tool sets (same `Arc`s the runner registers) so a call's OpenHuman
    /// `Tool` can be resolved for its generated-tool runtime context and its
    /// per-call permission level.
    tool_sets: Vec<Arc<Vec<Box<dyn Tool>>>>,
    channel: String,
}

/// Adapter from the host's [`ToolPolicy`](crate::agent::tool_policy::ToolPolicy)
/// (request + session context) to the harness `ToolCallPolicy` seam.
struct SessionToolPolicy {
    policy: Arc<dyn crate::agent::tool_policy::ToolPolicy>,
    session_id: String,
    channel: String,
    agent_definition_id: String,
}

#[async_trait]
impl ToolCallPolicy<OpenHumanRunContext> for SessionToolPolicy {
    fn name(&self) -> &str {
        policy_name
    }

    async fn check(
        &self,
        _ctx: &RunContext<OpenHumanRunContext>,
        call: &TaToolCall,
    ) -> PolicyDecision {
        use crate::agent::tool_policy::{ToolCallContext, ToolPolicyRequest};
        let decision = self.gate.check(ctx, &call, desktop_approval_disabled).await;
        let policy_name = self.gate.policy_name();
        if let Some(reason) = decision.blocking_reason() {
            let blocked_action = match &decision {
                ToolPolicyDecision::RequireApproval { .. } => "requires approval",
                ToolPolicyDecision::Deny { .. } => "denied",
                ToolPolicyDecision::Allow => "allowed",
            };
            crate::tools::registry::denials::record(
                call.name.as_str(),
                policy_name,
                blocked_action,
                reason,
            );
            tracing::debug!(
                tool = call.name.as_str(),
                policy = policy_name,
                action = blocked_action,
                reason = %reason,
                "[tinyagents::mw] tool blocked by policy"
            );
            let content = match &decision {
                ToolPolicyDecision::RequireApproval { .. } => PolicyDenial::ApprovalRequired {
                    tool: &call.name,
                    policy: policy_name,
                    reason,
                },
                _ => PolicyDenial::PolicyDenied {
                    tool: &call.name,
                    policy: policy_name,
                    reason,
                },
            }
            .render();
            return Ok(MiddlewareToolOutcome::Result(TaToolResult::error(content)));
        }

        // `use_skill`'s disclosure half (a `skill` with no `tool`) renders its
        // listing here, not in its own `execute`, because only the middleware
        // holds the session — and a listing that disagrees with the session is
        // a menu the model cannot order from. The execution half falls through:
        // `render_skill_for_session` answers `None` for it.
        //
        // Placed AFTER both gates on purpose. Rendering before them would let a
        // `use_skill` that the session forbids, or that the policy denies or
        // holds for approval, still hand back a full pack listing — the gates
        // would be advisory for this one tool.
        if call.name == tinyagents_harness::tool::packs::USE_SKILL {
            if let Some(result) = self.render_skill_for_session(&call) {
                return Ok(MiddlewareToolOutcome::Result(result));
            }
        }

        next.run(ctx, state, call).await
    }
}
