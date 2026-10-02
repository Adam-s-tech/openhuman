//! JSON-RPC request/response models for the `threads` namespace.
//!
//! Moved verbatim from the v1 `memory::rpc_models` when memory v2 replaced
//! that module: the serde shapes, field names and defaults are unchanged, so
//! nothing on the wire moved. These types describe chat threads and their
//! messages, which are thread persistence (`threads::store`), not memory.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Standard error structure for API responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    /// A machine-readable error code.
    pub code: String,
    /// A human-readable error message.
    pub message: String,
    /// Optional additional error details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// Pagination metadata for list-based responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationMeta {
    /// Maximum number of items requested.
    pub limit: usize,
    /// Number of items skipped.
    pub offset: usize,
    /// Total number of items available in the backend.
    pub count: usize,
}

/// General metadata included in all API envelopes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiMeta {
    /// Unique identifier for the request.
    pub request_id: String,
    /// Time taken to process the request in seconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_seconds: Option<f64>,
    /// Whether the response was served from a cache.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached: Option<bool>,
    /// Optional counts of various items (e.g., by category).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub counts: Option<BTreeMap<String, usize>>,
    /// Optional pagination information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination: Option<PaginationMeta>,
}

/// Generic envelope for all API responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiEnvelope<T> {
    /// The actual payload of the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    /// Error information if the request failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ApiError>,
    /// Metadata about the request and response.
    pub meta: ApiMeta,
}

/// An empty request body for methods that don't require parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EmptyRequest {}

/// Request to create a new conversation thread.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateConversationThreadRequest {
    #[serde(default)]
    pub labels: Option<Vec<String>>,
    #[serde(default)]
    pub personality_id: Option<String>,
}

/// Summary information for a workspace-backed conversation thread.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationThreadSummary {
    pub id: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chat_id: Option<i64>,
    pub is_active: bool,
    pub message_count: usize,
    pub last_message_at: String,
    pub created_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_thread_id: Option<String>,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub personality_id: Option<String>,
}

/// A single persisted conversation message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationMessageRecord {
    pub id: String,
    pub content: String,
    #[serde(rename = "type")]
    pub message_type: String,
    #[serde(default)]
    pub extra_metadata: serde_json::Value,
    pub sender: String,
    pub created_at: String,
}

/// Request to create or update a thread in workspace storage.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpsertConversationThreadRequest {
    pub id: String,
    pub title: String,
    pub created_at: String,
    #[serde(default)]
    pub parent_thread_id: Option<String>,
    #[serde(default)]
    pub labels: Option<Vec<String>>,
    #[serde(default)]
    pub personality_id: Option<String>,
}

/// Request to update labels for a conversation thread.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateConversationThreadLabelsRequest {
    pub thread_id: String,
    pub labels: Vec<String>,
}

/// Request to set a user-specified title on a conversation thread.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateConversationThreadTitleRequest {
    pub thread_id: String,
    pub title: String,
}

/// Response payload for thread list operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationThreadsListResponse {
    pub threads: Vec<ConversationThreadSummary>,
    pub count: usize,
}

/// Request to fetch messages for a specific thread.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConversationMessagesRequest {
    pub thread_id: String,
}

/// Response payload for message list operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationMessagesResponse {
    pub messages: Vec<ConversationMessageRecord>,
    pub count: usize,
}

/// Request to append a message to a thread.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AppendConversationMessageRequest {
    pub thread_id: String,
    pub message: ConversationMessageRecord,
}

/// Request to generate or refresh a thread title after the first exchange.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GenerateConversationThreadTitleRequest {
    pub thread_id: String,
    #[serde(default)]
    pub assistant_message: Option<String>,
}

/// Request to patch a persisted message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateConversationMessageRequest {
    pub thread_id: String,
    pub message_id: String,
    #[serde(default)]
    pub extra_metadata: Option<serde_json::Value>,
}

/// Request to delete a thread and its message log.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeleteConversationThreadRequest {
    pub thread_id: String,
    pub deleted_at: String,
}

/// Response payload for single-thread deletion.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteConversationThreadResponse {
    pub deleted: bool,
}

/// Response payload for purging all workspace-backed conversations.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PurgeConversationThreadsResponse {
    pub messages_deleted: usize,
    pub agent_threads_deleted: usize,
    pub agent_messages_deleted: usize,
}

/// Request payload for `openhuman.list_documents`.
