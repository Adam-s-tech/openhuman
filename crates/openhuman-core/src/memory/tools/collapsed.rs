//! `memory` — the whole agent-facing memory surface as one four-action tool:
//! `memory(action: "ask" | "keyword_search" | "learn" | "forget", text: "...")`.
//!
//! * `ask` — a question in plain language; answered by the hybrid
//!   (keyword + semantic) search over stored chunks.
//! * `keyword_search` — keywords or a phrase; the lexical `memory_recall` search.
//! * `learn` — one explicit learning to keep, saved as a durable fact.
//! * `forget` — delete one stored memory by its key.
//!
//! The model sees one verb per intent and one text argument, instead of the
//! eleven schemas (`memory_store`, `memory_recall`, `memory_hybrid_search`, …)
//! this replaced. Those tools stay registered and `Hidden`, dispatchable by
//! name for a replayed transcript or a curated belt that lists one (the memory
//! agent's `memory_tree`, `memory_doctor`, `memory_flavour`, …); they are just
//! not part of this tool's surface.
//!
//! # `memory_tree` is deliberately NOT folded in
//!
//! It dispatches eight operations on a `mode` field over the ingested
//! email/chat/document tree, a different subsystem with a different storage
//! model. Two tools that each dispatch once beat one that dispatches twice.
//!
//! # Permissions
//!
//! `permission_level_with_args` resolves the member's own level from the
//! action; the argument-free `permission_level` reports the strictest any
//! member requires, so an argument-less caller over-restricts rather than
//! under-. See `tinytools::collapse`.
//!
//! **Pre-existing, and left alone:** `memory_store` does not override
//! `permission_level`, so a write inherits the `ReadOnly` default; it gates
//! internally through its own `SecurityPolicy` + `ToolOperation` check.
//! Collapsing reproduces that and does not correct it — raising it would change
//! which turns get parked for approval, a product decision.

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{json, Value};

use super::forget::MemoryForgetTool;
use super::recall::MemoryRecallTool;
use super::search::MemoryHybridSearchTool;
use super::store::MemoryStoreTool;
use crate::config::Config;
use crate::security::policy::SecurityPolicy;
use tinytools::collapse::{
    any_external_effect, resolve, strictest_permission, unknown_action_message, CollapsedAction,
};
use tinytools::{PermissionLevel, Tool, ToolCallOptions, ToolResult};

#[cfg(test)]
use tinytools::ToolExposure;

/// The advertised name.
pub const MEMORY_TOOL_NAME: &str = "memory";

/// Answer a question in plain language (hybrid keyword + semantic search).
pub const ACTION_ASK: &str = "ask";
/// Search by keywords or a phrase (lexical recall).
pub const ACTION_KEYWORD_SEARCH: &str = "keyword_search";
/// Save one explicit learning as a durable fact.
pub const ACTION_LEARN: &str = "learn";
/// Delete one stored memory.
pub const ACTION_FORGET: &str = "forget";

pub struct MemoryTool {
    store: MemoryStoreTool,
    recall: MemoryRecallTool,
    forget: MemoryForgetTool,
    hybrid_search: MemoryHybridSearchTool,
}

impl MemoryTool {
    pub fn new(_config: Arc<Config>, security: Arc<SecurityPolicy>) -> Self {
        Self {
            store: MemoryStoreTool::new(Arc::clone(&security)),
            recall: MemoryRecallTool::new(),
            forget: MemoryForgetTool::new(security),
            hybrid_search: MemoryHybridSearchTool::default(),
        }
    }

    /// The action table, in the order it is advertised, filtered by memory
    /// capability: one tool cannot be dropped for one capability, so an
    /// action the active memory driver does not serve is left out of the enum
    /// rather than inviting a call that always errors.
    fn actions(&self) -> Vec<CollapsedAction<'_>> {
        self.all_actions()
            .into_iter()
            .filter(|entry| {
                crate::core::all::capability_allowed(crate::tools::ops::tool_capability(
                    entry.tool.name(),
                ))
            })
            .collect()
    }

    /// Every action this tool can serve, before capability filtering.
    fn all_actions(&self) -> Vec<CollapsedAction<'_>> {
        vec![
            CollapsedAction {
                action: ACTION_ASK,
                tool: &self.hybrid_search,
            },
            CollapsedAction {
                action: ACTION_KEYWORD_SEARCH,
                tool: &self.recall,
            },
            CollapsedAction {
                action: ACTION_LEARN,
                tool: &self.store,
            },
            CollapsedAction {
                action: ACTION_FORGET,
                tool: &self.forget,
            },
        ]
    }
}

/// Translate the tool's `{action, text, namespace?, limit?}` call into the
/// member tool's own argument shape.
fn member_args(action: &str, args: &Value) -> Result<Value, String> {
    let text = args
        .get("text")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .ok_or_else(|| "memory: `text` is required and cannot be empty".to_string())?;
    let namespace = args.get("namespace").cloned();
    let limit = args.get("limit").cloned();
    let mut out = serde_json::Map::new();
    match action {
        ACTION_ASK => {
            out.insert("query".into(), json!(text));
            // The hybrid search requires a namespace; default to the
            // assistant's own memory like every other memory tool.
            out.insert(
                "namespace".into(),
                namespace.unwrap_or_else(|| {
                    json!(crate::agent::tinyagents::host::agent_memory::DEFAULT_AGENT_MEMORY_NAMESPACE)
                }),
            );
        }
        ACTION_KEYWORD_SEARCH => {
            out.insert("query".into(), json!(text));
            if let Some(ns) = namespace {
                out.insert("namespace".into(), ns);
            }
        }
        ACTION_FORGET => {
            // `text` is the memory's key, as printed in every search result.
            out.insert("key".into(), json!(text));
            out.insert(
                "namespace".into(),
                namespace.unwrap_or_else(|| {
                    json!(crate::agent::tinyagents::host::agent_memory::DEFAULT_AGENT_MEMORY_NAMESPACE)
                }),
            );
            return Ok(Value::Object(out));
        }
        ACTION_LEARN => {
            out.insert("content".into(), json!(text));
            if let Some(ns) = namespace {
                out.insert("namespace".into(), ns);
            }
            // `limit` means nothing to a write.
            return Ok(Value::Object(out));
        }
        _ => return Err(format!("memory: unknown action `{action}`")),
    }
    if let Some(limit) = limit {
        out.insert("limit".into(), limit);
    }
    Ok(Value::Object(out))
}

#[async_trait]
impl Tool for MemoryTool {
    fn name(&self) -> &str {
        MEMORY_TOOL_NAME
    }

    fn description(&self) -> &str {
        "The user's long-term memory. `ask`: a question in plain language \
         (start here). `keyword_search`: keywords or a phrase when you know \
         the words. `learn`: one explicit, durable thing worth remembering. \
         `forget`: delete one memory; `text` is its key, as shown in a search \
         result. Otherwise `text` is the question, the keywords or the learning. For ingested \
         email, chat and documents use the separate `memory_tree` tool."
    }

    fn parameters_schema(&self) -> Value {
        let actions: Vec<&str> = self.actions().iter().map(|a| a.action).collect();
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": actions,
                    "description": "`ask` a question, `keyword_search` by words, `learn` a fact, or `forget` one by key."
                },
                "text": {
                    "type": "string",
                    "description": "The question, the keywords, the learning to keep, or (for `forget`) the memory key."
                },
                "limit": {
                    "type": "integer",
                    "description": "Max results for `ask` / `keyword_search`."
                },
                "namespace": {
                    "type": "string",
                    "description": "Optional. Defaults to the assistant's own memory."
                }
            },
            "required": ["action", "text"]
        })
    }

    fn permission_level(&self) -> PermissionLevel {
        strictest_permission(&self.actions())
    }

    fn permission_level_with_args(&self, args: &Value) -> PermissionLevel {
        // `forget` is destructive and `recall` is read-only; reporting one
        // level for both would either gate every read or let a delete through
        // on a read's clearance. Unknown/missing actions take the strictest.
        let actions = self.actions();
        args.get("action")
            .and_then(Value::as_str)
            .and_then(|action| resolve(&actions, action))
            .map(|entry| entry.tool.permission_level_with_args(args))
            .unwrap_or_else(|| strictest_permission(&actions))
    }

    fn external_effect(&self) -> bool {
        any_external_effect(&self.actions())
    }

    fn supports_markdown(&self) -> bool {
        true
    }

    async fn execute(&self, args: Value) -> anyhow::Result<ToolResult> {
        self.execute_with_options(args, ToolCallOptions::default())
            .await
    }

    async fn execute_with_options(
        &self,
        args: Value,
        options: ToolCallOptions,
    ) -> anyhow::Result<ToolResult> {
        let actions = self.actions();
        let requested = args.get("action").and_then(Value::as_str);
        let Some(entry) = requested.and_then(|action| resolve(&actions, action)) else {
            return Ok(ToolResult::error(unknown_action_message(
                &actions, requested,
            )));
        };
        let member = match member_args(entry.action, &args) {
            Ok(member) => member,
            Err(message) => return Ok(ToolResult::error(message)),
        };
        tracing::debug!(action = %entry.action, "[tool][memory] dispatch");
        entry.tool.execute_with_options(member, options).await
    }
}

#[cfg(test)]
#[path = "collapsed_tests.rs"]
mod tests;
