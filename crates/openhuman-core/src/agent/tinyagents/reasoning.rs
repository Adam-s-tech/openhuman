//! The user's reasoning ("thinking") choice for an agent turn, resolved into
//! the provider-neutral [`ReasoningConfig`] the harness attaches to every
//! model request (`RunPolicy::default_reasoning`).
//!
//! The source is the turn's effective config: `runtime.reasoning_effort`
//! (a chat thread's per-turn choice is written there by `web_chat`), falling
//! back to `runtime.reasoning_enabled = false` meaning "no reasoning". Nothing
//! set means the provider keeps its own default.

use tinyinference_llm::model::{ReasoningConfig, ReasoningEffort};

use crate::config::Config;

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
