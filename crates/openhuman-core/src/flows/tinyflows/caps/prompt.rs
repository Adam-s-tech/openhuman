//! Turning a node's request into a completion, and a completion back into a
//! node result.
//!
//! Message assembly, the `input_context` carrier and its size cap, the
//! structured-output contract, and the tolerant JSON extraction a model reply
//! needs when it wraps its object in prose or a fenced block.

#![allow(unused_imports)]

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{json, Value};
use tinyflows::caps::*;
use tinyflows::error::{EngineError, Result};

use super::*;
use crate::agent::messages::ChatMessage;
use crate::config::Config;
use crate::inference::provider::{is_raw_passthrough_model, UsageInfo};

/// Maps a `UsageInfo` (not `Serialize`) into a JSON value field-by-field, so
/// [`OpenHumanLlm::complete`] can surface it in its response `Value` without
/// requiring an upstream `Serialize` impl change.
pub(crate) fn usage_to_json(usage: &Option<UsageInfo>) -> Value {
    match usage {
        None => Value::Null,
        Some(u) => json!({
            "input_tokens": u.input_tokens,
            "output_tokens": u.output_tokens,
            "context_window": u.context_window,
            "cached_input_tokens": u.cached_input_tokens,
            "cache_creation_tokens": u.cache_creation_tokens,
            "reasoning_tokens": u.reasoning_tokens,
            "charged_amount_usd": u.charged_amount_usd,
        }),
    }
}

pub(crate) fn model_response_to_completion_value(
    response: &tinyinference_llm::model::ModelResponse,
) -> Value {
    json!({
        "text": response.text(),
        "tool_calls": response
            .tool_calls()
            .iter()
            .map(crate::agent::tinyagents::ta_call_to_oh_call)
            .collect::<Vec<_>>(),
        "usage": usage_to_json(
            &crate::agent::tinyagents::model::usage_info_from_response(response)
        ),
        "reasoning_content": crate::agent::tinyagents::reasoning_from_content(
            &response.message.content
        ),
    })
}

#[cfg(test)]
#[path = "prompt_tests.rs"]
mod tests;

/// Select the model an `agent` node completion actually runs on.
///
/// `resolved_model` is what [`create_chat_provider`] returned for the node's
/// mapped workload role. A node may instead pin a **raw/BYOK** model id
/// (e.g. `claude-opus-4`) that [`role_for_model_tier`] collapsed to the `chat`
/// role — in that case the pinned id, not the role default, is the model the
/// user selected, so it is forwarded verbatim (issue #4598). Managed tiers and
/// every `hint:*` alias fall through to `resolved_model` unchanged.
pub(crate) fn resolve_completion_model(node_model: Option<&str>, resolved_model: String) -> String {
    match node_model {
        Some(pinned) if is_raw_passthrough_model(pinned) => {
            tracing::debug!(
                target: "flows",
                raw_model = pinned,
                "[flows] llm.complete: forwarding raw/BYOK node model verbatim (not a managed tier)"
            );
            pinned.to_string()
        }
        _ => resolved_model,
    }
}
