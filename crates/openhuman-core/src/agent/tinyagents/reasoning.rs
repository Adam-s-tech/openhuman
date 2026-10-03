//! The user's reasoning ("thinking") choice for an agent turn, resolved into
//! the provider-neutral [`ReasoningConfig`] the harness attaches to every
//! model request (`RunPolicy::default_reasoning`).
//!
//! Precedence, highest first:
//!
//! 1. the thread's own choice, recorded by `channel_web_chat`'s
//!    `reasoning_effort` param ([`apply_requested_effort`]) — read at turn
//!    start, so changing it never evicts the thread's warm session;
//! 2. `runtime.reasoning_effort` in the session's effective config;
//! 3. `runtime.reasoning_enabled = false`, meaning "no reasoning".
//!
//! Nothing set means the provider keeps its own default.

use std::collections::HashMap;
use std::sync::OnceLock;

use parking_lot::Mutex;
use tinyinference_llm::model::{ReasoningConfig, ReasoningEffort};

use crate::config::Config;

fn thread_efforts() -> &'static Mutex<HashMap<String, ReasoningEffort>> {
    static EFFORTS: OnceLock<Mutex<HashMap<String, ReasoningEffort>>> = OnceLock::new();
    EFFORTS.get_or_init(|| Mutex::new(HashMap::new()))
}

/// The effort a thread's user picked, if any.
pub(crate) fn thread_effort(thread_id: &str) -> Option<ReasoningEffort> {
    thread_efforts().lock().get(thread_id).copied()
}

/// Records (`Some`) or clears (`None`) a thread's chosen effort.
pub(crate) fn set_thread_effort(thread_id: &str, effort: Option<ReasoningEffort>) {
    let mut map = thread_efforts().lock();
    let previous = match effort {
        Some(effort) => map.insert(thread_id.to_string(), effort),
        None => map.remove(thread_id),
    };
    if previous != effort {
        log::info!(
            "[agent][reasoning] thread reasoning effort changed thread_id={thread_id} {:?} -> {:?}",
            previous.map(ReasoningEffort::as_str),
            effort.map(ReasoningEffort::as_str)
        );
    }
}

/// Applies a chat request's `reasoning_effort` param to its thread.
///
/// Absent leaves the thread's choice as it was (older clients and the socket
/// path never send it); `""`/`default`/`auto` clears it back to config; any
/// other value must parse, otherwise the request is rejected so a typo'd
/// client cannot silently run at the wrong level.
pub(crate) fn apply_requested_effort(thread_id: &str, raw: Option<&str>) -> Result<(), String> {
    let Some(raw) = raw else {
        return Ok(());
    };
    let trimmed = raw.trim();
    if trimmed.is_empty()
        || trimmed.eq_ignore_ascii_case("default")
        || trimmed.eq_ignore_ascii_case("auto")
    {
        set_thread_effort(thread_id, None);
        return Ok(());
    }
    let effort = parse_reasoning_effort(trimmed)
        .ok_or_else(|| format!("unknown reasoning_effort '{trimmed}'"))?;
    set_thread_effort(thread_id, Some(effort));
    Ok(())
}

/// The reasoning config for the turn `ctx` is about to run, or `None` for the
/// provider default.
///
/// Only a root turn (spawn depth 0) follows the user's thinking level; a
/// sub-agent runs at its provider default, since the user picked a level for
/// the conversation, not for every delegated helper. `config` is the session
/// config a hosted root turn carries (`OpenHumanHostBase::config`).
pub(crate) fn turn_reasoning_for(
    ctx: &crate::agent::tinyagents::host::OpenHumanRunContext,
    config: Option<&Config>,
    _max_output_tokens: Option<u32>,
) -> Option<ReasoningConfig> {
    if ctx.spawn_depth > 0 {
        return None;
    }
    let reasoning = turn_reasoning(ctx.thread_id.as_deref(), config);
    if let Some(reasoning) = reasoning.as_ref() {
        log::debug!(
            "[agent][reasoning] turn reasoning effort={:?} budget_tokens={:?} thread_id={:?}",
            reasoning.effort,
            reasoning.budget_tokens,
            ctx.thread_id
        );
    }
    reasoning
}

/// The reasoning config one agent turn should request: the thread's own
/// choice, else [`reasoning_for_config`].
pub(crate) fn turn_reasoning(
    thread_id: Option<&str>,
    config: Option<&Config>,
) -> Option<ReasoningConfig> {
    if let Some(effort) = thread_id.and_then(thread_effort) {
        return Some(ReasoningConfig::effort(effort));
    }
    config.and_then(reasoning_for_config)
}

/// Parses a user-facing effort name. Accepts the wire tokens plus the aliases
/// `off`/`disabled` (none) and `max`/`maximum` (xhigh), case-insensitively.
pub(crate) fn parse_reasoning_effort(raw: &str) -> Option<ReasoningEffort> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "none" | "off" | "disabled" => Some(ReasoningEffort::None),
        "minimal" => Some(ReasoningEffort::Minimal),
        "low" => Some(ReasoningEffort::Low),
        "medium" | "med" => Some(ReasoningEffort::Medium),
        "high" => Some(ReasoningEffort::High),
        "xhigh" | "max" | "maximum" => Some(ReasoningEffort::XHigh),
        _ => None,
    }
}

/// The reasoning config an agent turn on `config` should request, or `None`
/// to leave the provider default alone.
pub(crate) fn reasoning_for_config(config: &Config) -> Option<ReasoningConfig> {
    if let Some(raw) = config
        .runtime
        .reasoning_effort
        .as_deref()
        .map(str::trim)
        .filter(|raw| !raw.is_empty())
    {
        return match parse_reasoning_effort(raw) {
            Some(effort) => Some(ReasoningConfig::effort(effort)),
            None => {
                log::warn!(
                    "[agent][reasoning] ignoring unknown runtime.reasoning_effort {raw:?}; \
                     using the provider default"
                );
                None
            }
        };
    }
    if config.runtime.reasoning_enabled == Some(false) {
        return Some(ReasoningConfig::effort(ReasoningEffort::None));
    }
    None
}

#[cfg(test)]
#[path = "reasoning_tests.rs"]
mod tests;
