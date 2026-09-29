//! Engine-neutral fallbacks for the user-facing memory RPCs.
//!
//! Remote memory engines implement only the mandatory families
//! (`MemoryCore`, `MemoryRecall`, `MemoryPortability`). The RPCs below used to
//! require an optional family (`documents`, `retrieval`) and errored out when
//! bound to such an engine. Each helper here answers the same question from the
//! mandatory surface, so the core memory experience works on any engine:
//!
//! - namespaces: `MemoryCore::namespaces`
//! - ranked recall: `MemoryRecall::recall`
//! - recency recall: `MemoryCore::list`, newest first
//! - document list: `MemoryCore::list`, mapped to document summaries
//!
//! Graph and tree stay capability-gated; [`unsupported_family`] builds the
//! stable error they return, and [`is_unsupported_family_error`] is the
//! matching detector.

use chrono::{DateTime, Utc};
use serde_json::{json, Value};

use crate::memory::api::provider::{MemoryCore, MemoryRecall};
use crate::memory::api::recall::OwnedRecallOpts;
use crate::memory::api::types::{
    MemoryEntry, MemoryItemKind, NamespaceMemoryHit, RetrievalScoreBreakdown,
};
use crate::memory::guard::MemoryGuard;

/// Stable prefix of every capability-gate error. The UI matches on
/// `does not support the <family> family`; never reword it.
pub(crate) const UNSUPPORTED_FAMILY_PREFIX: &str = "memory driver does not support the ";

/// The capability-gate error for `family`.
pub(crate) fn unsupported_family(family: &str) -> String {
    format!("{UNSUPPORTED_FAMILY_PREFIX}{family} family")
}

/// Whether `message` is a capability-gate error built by [`unsupported_family`].
#[allow(dead_code)]
pub(crate) fn is_unsupported_family_error(message: &str) -> bool {
    message.contains(UNSUPPORTED_FAMILY_PREFIX) && message.contains(" family")
}

fn entry_epoch(entry: &MemoryEntry) -> f64 {
    DateTime::parse_from_rfc3339(&entry.timestamp)
        .map(|t| t.with_timezone(&Utc).timestamp() as f64)
        .unwrap_or(0.0)
}

/// Map a mandatory-surface entry into the retrieval hit shape the RPC
/// responses are built from.
pub(crate) fn entry_to_hit(entry: MemoryEntry, namespace: &str) -> NamespaceMemoryHit {
    let updated_at = entry_epoch(&entry);
    let score = entry.score.unwrap_or(0.0);
    NamespaceMemoryHit {
        id: entry.id,
        kind: MemoryItemKind::Kv,
        namespace: entry.namespace.unwrap_or_else(|| namespace.to_string()),
        title: Some(entry.key.clone()),
        key: entry.key,
        content: entry.content,
        category: entry.category.to_string(),
        source_type: Some("recall".to_string()),
        updated_at,
        score,
        score_breakdown: RetrievalScoreBreakdown {
            keyword_relevance: 0.0,
            // A remote engine reports one relevance score. It stands in for the
            // vector component so the relevance floors that gate auto-injected
            // context (`select_notes`, `recall_by_vector_over`) can still apply.
            vector_similarity: score,
            graph_relevance: 0.0,
            episodic_relevance: 0.0,
            freshness: 0.0,
            final_score: score,
        },
        document_id: None,
        chunk_id: None,
        supporting_relations: Vec::new(),
        taint: entry.taint,
    }
}

/// Namespace names via `MemoryCore::namespaces`.
pub(crate) async fn namespace_names(guard: &MemoryGuard) -> Result<Vec<String>, String> {
    log::debug!("[memory:fallback] list_namespaces via MemoryCore::namespaces");
    let summaries = guard
        .namespaces()
        .await
        .map_err(|error| error.to_string())?;
    Ok(summaries.into_iter().map(|s| s.namespace).collect())
}

/// Ranked hits via `MemoryRecall::recall`, scoped to `namespace`.
pub(crate) async fn recall_hits<P: MemoryRecall + ?Sized>(
    guard: &P,
    namespace: &str,
    query: &str,
    limit: usize,
) -> Result<Vec<NamespaceMemoryHit>, String> {
    log::debug!(
        "[memory:fallback] ranked recall via MemoryRecall::recall namespace={namespace} limit={limit}"
    );
    let opts = OwnedRecallOpts {
        namespace: Some(namespace.to_string()),
        ..Default::default()
    };
    let entries = guard
        .recall(query, limit, &opts, None)
        .await
        .map_err(|error| error.to_string())?;
    Ok(entries
        .into_iter()
        .take(limit)
        .map(|entry| entry_to_hit(entry, namespace))
        .collect())
}

/// Most recently updated hits via `MemoryCore::list`, newest first.
///
/// There is no query, so a ranked `recall` cannot answer this: the contract
/// says an empty query yields nothing.
pub(crate) async fn recent_hits(
    guard: &MemoryGuard,
    namespace: &str,
    limit: usize,
) -> Result<Vec<NamespaceMemoryHit>, String> {
    log::debug!(
        "[memory:fallback] recency recall via MemoryCore::list namespace={namespace} limit={limit}"
    );
    let mut entries = guard
        .list(Some(namespace), None, None)
        .await
        .map_err(|error| error.to_string())?;
    entries.sort_by(|a, b| entry_epoch(b).total_cmp(&entry_epoch(a)));
    entries.truncate(limit);
    Ok(entries
        .into_iter()
        .map(|entry| entry_to_hit(entry, namespace))
        .collect())
}

/// Document list in the `{"documents": [...]}` shape `list_documents`
/// returns, built from `MemoryCore::list`.
pub(crate) async fn document_list(
    guard: &MemoryGuard,
    namespace: Option<&str>,
) -> Result<Value, String> {
    log::debug!("[memory:fallback] doc_list via MemoryCore::list namespace={namespace:?}");
    let entries = guard
        .list(namespace, None, None)
        .await
        .map_err(|error| error.to_string())?;
    let documents = entries
        .into_iter()
        .map(|entry| {
            let ts = entry_epoch(&entry);
            json!({
                "document_id": entry.id,
                "namespace": entry.namespace.or_else(|| namespace.map(str::to_string)).unwrap_or_default(),
                "key": entry.key,
                "title": entry.key,
                "source_type": "memory",
                "priority": "medium",
                "created_at": ts,
                "updated_at": ts,
            })
        })
        .collect::<Vec<_>>();
    Ok(json!({ "documents": documents }))
}

#[cfg(test)]
#[path = "fallback_tests.rs"]
mod tests;
