//! Entry points for the lifecycle moments that are not tool calls.
//!
//! Tool events reach the engine through
//! [`bridge`](super::bridge) — the harness already had a seam there. The
//! moments in this module have no such seam, so each is a small function a call
//! site invokes directly. Each one is cheap when nothing is configured: it asks
//! [`HookEngine::has_hooks`](super::engine::HookEngine::has_hooks) first and
//! returns without building an envelope.

use std::path::PathBuf;
use std::time::Duration;

use tinyagents_runtime::command_hooks::context::{
    build_input, set_host_context, HostContext, TurnIdentity,
};
use tinyagents_runtime::command_hooks::types::{
    HookEvent, HookPayload, PromptPayload, SubagentPayload,
};

/// Bring the hook system up: resolve host facts, read every `hooks.json`, and
/// install the harness bridge.
///
/// Safe to call again — a reload replaces the whole config and re-registers the
/// bridge by name rather than appending a second copy.
pub async fn init(config: &crate::config::schema::Config) {
    let settings = &config.hooks;
    set_host_context(HostContext {
        workspace_roots: workspace_roots(config),
        version: env!("CARGO_PKG_VERSION").to_string(),
    });

    if !settings.enabled {
        super::bridge::ConfiguredHookBridge::uninstall();
        super::host::engine()
            .install(tinyagents_runtime::command_hooks::HookConfig::default())
            .await;
        log::info!("[hooks] disabled by configuration");
        return;
    }

    super::host::engine()
        .set_default_timeout(Duration::from_secs(settings.default_timeout_secs))
        .await;
    let loaded = super::host::engine()
        .reload(
            Some(config.action_dir.clone()),
            Some(config.workspace_dir.clone()),
        )
        .await;
    if loaded.is_empty() {
        // Nothing configured: keep the bridge out of the harness entirely so an
        // unconfigured host pays not even a task-local lookup per tool call.
        super::bridge::ConfiguredHookBridge::uninstall();
        return;
    }
    super::bridge::ConfiguredHookBridge::install();
}

fn workspace_roots(config: &crate::config::schema::Config) -> Vec<PathBuf> {
    let mut roots = vec![config.action_dir.clone()];
    if let Some(turn_root) = crate::agent::turn_workspace::current() {
        if !roots.contains(&turn_root) {
            roots.push(turn_root);
        }
    }
    roots
}

/// The verdict on a submitted prompt.
#[derive(Debug, Clone)]
pub enum PromptVerdict {
    /// Send it, optionally with context a hook wants prepended.
    Submit {
        /// Text a hook asked to add to the turn's context.
        additional_context: Option<String>,
    },
    /// Do not send it. The string is what the user should be told.
    Block(String),
}

/// Fire `beforeSubmitPrompt`.
///
/// Blocking is expressed two ways in Cursor — `continue: false` and
/// `permission: "deny"` — and both are honoured, because a hook author reaching
/// for the one they remember should not silently get a prompt that went through
/// anyway.
pub async fn prompt_submitted(
    identity: TurnIdentity,
    prompt: &str,
    attachments: Vec<String>,
) -> PromptVerdict {
    let engine = super::host::engine();
    if !engine.has_hooks(HookEvent::BeforeSubmitPrompt).await {
        return PromptVerdict::Submit {
            additional_context: None,
        };
    }
    let input = build_input(
        HookEvent::BeforeSubmitPrompt,
        identity,
        HookPayload::Prompt(PromptPayload {
            prompt: prompt.to_string(),
            attachments,
        }),
    );
    let outcome = engine.dispatch(HookEvent::BeforeSubmitPrompt, input).await;
    if outcome.is_deny() || outcome.output.continue_ == Some(false) {
        let reason = outcome
            .output
            .user_message
            .clone()
            .or_else(|| outcome.output.agent_message.clone())
            .unwrap_or_else(|| "a configured hook blocked this prompt".to_string());
        return PromptVerdict::Block(reason);
    }
    PromptVerdict::Submit {
        additional_context: outcome.output.additional_context,
    }
}

/// Fire `subagentStart`. `Err` carries the reason the child must not run.
pub async fn subagent_starting(
    identity: TurnIdentity,
    subagent_type: &str,
    task: &str,
) -> Result<(), String> {
    let engine = super::host::engine();
    if !engine.has_hooks(HookEvent::SubagentStart).await {
        return Ok(());
    }
    let input = build_input(
        HookEvent::SubagentStart,
        identity.clone(),
        HookPayload::Subagent(SubagentPayload {
            subagent_type: subagent_type.to_string(),
            task: task.to_string(),
            parent_conversation_id: identity.conversation_id,
            status: None,
            duration_ms: None,
        }),
    );
    let outcome = engine.dispatch(HookEvent::SubagentStart, input).await;
    match outcome.denial_reason() {
        Some(reason) => Err(reason.to_string()),
        None => Ok(()),
    }
}
