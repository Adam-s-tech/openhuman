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
//! - recency recall: at most [`RECENT_MAX_PAGES`] bounded `export_page` calls
//!   filtered to the namespace, newest first
//! - document list: `MemoryCore::list` for one namespace, or a bounded scan of
//!   at most [`DOC_LIST_MAX_NAMESPACES`] namespaces, capped at
//!   [`DOC_LIST_CAP`] documents with a `truncated` flag
//!
//! Every helper is bounded in provider calls and records: none ever lists every
//! scope of a remote engine. The proper fix is a bounded
//! `recent(namespace, limit)` on the tinymemory contract, an upstream follow-up
//! this module will switch to when it exists.
//!
//! Graph and tree stay capability-gated; [`unsupported_family`] builds the
//! stable error they return, and [`is_unsupported_family_error`] is the
//! matching detector.

use chrono::{DateTime, Utc};
use serde_json::{json, Value};

use crate::memory::api::provider::{MemoryCore, MemoryPortability, MemoryRecall};
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
pub(crate) fn is_unsupported_family_error(message: &str) -> bool {
    message.contains(UNSUPPORTED_FAMILY_PREFIX) && message.contains(" family")
}

/// Route an error from a memory RPC handler through the engine error
/// vocabulary when the bound engine is remote, so a hosted 402 reads
/// `INSUFFICIENT_CREDITS:` and a rejected session `SESSION_EXPIRED:`. Local
/// engines and capability-gate errors pass through unchanged.
pub(crate) fn classify_for_engine(external: bool, message: String) -> String {
    if !external || is_unsupported_family_error(&message) {
        return message;
    }
    super::engine::classify_engine_message(&message)
}

/// [`classify_for_engine`] for the engine bound to the ambient context. With no
/// context the message is classified on its content alone: the hosted
/// dialect's `[USER_INSUFFICIENT_CREDITS]` / `[UNAUTHORIZED]` codes only ever
/// come from a remote engine.
pub(crate) fn classify_rpc_error(message: String) -> String {
    let external = crate::core::runtime::context::CoreContext::current()
        .and_then(|ctx| ctx.memory().ok())
        .is_none_or(|guard| {
            guard.policy().class() == crate::core::subsystem::DriverClass::External
        });
    classify_for_engine(external, message)
}

/// Records fetched per page by [`recent_hits`], at most.
pub(crate) const RECENT_PAGE_CAP: usize = 200;
/// Pages [`recent_hits`] reads, at most.
pub(crate) const RECENT_MAX_PAGES: usize = 2;
/// Documents [`document_list`] returns, at most.
pub(crate) const DOC_LIST_CAP: usize = 200;
/// Namespaces [`document_list`] scans when none is given, at most.
pub(crate) const DOC_LIST_MAX_NAMESPACES: usize = 10;

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

/// Whether `hits` were ranked without anything measured: each has a positive
/// final score and no signal at all — no similarity, keyword, graph, episodic
/// or freshness.
///
/// That is the mark a driver ranking without scoring leaves (hosted CortexDB's
/// retrieval family scores a hit by its rank and reports no signal), and a
/// weighted sum of signals cannot produce it, so an engine that scores is
/// never read as one. A similarity floor cannot tell such hits apart, so a
/// caller treats them as it treats [`ranked_recall`]'s unscored ones. An empty
/// list is not rank-only: there is nothing to keep either way.
pub(crate) fn rank_only(hits: &[NamespaceMemoryHit]) -> bool {
    !hits.is_empty()
        && hits.iter().all(|hit| {
            let signals = &hit.score_breakdown;
            signals.final_score > 0.0
                && signals.vector_similarity == 0.0
                && signals.keyword_relevance == 0.0
                && signals.graph_relevance == 0.0
                && signals.episodic_relevance == 0.0
                && signals.freshness == 0.0
        })
}

/// What [`ranked_recall`] answered: the hits, and whether the engine scored
/// them.
pub(crate) struct RankedRecall {
    /// The engine's hits, in its order.
    pub(crate) hits: Vec<NamespaceMemoryHit>,
    /// Whether any hit carried the engine's own score. `false` for an engine
    /// whose recall is ranked but unscored — hosted CortexDB answers no score
    /// at all — whose hits [`entry_to_hit`] then reads as 0.0.
    pub(crate) scored: bool,
}

/// Ranked hits via `MemoryRecall::recall`, scoped to `namespace`, and whether
/// the engine scored them.
///
/// A caller that floors on similarity has to know the difference: an
/// unscored engine's 0.0 means "no score", not "irrelevant", and flooring it
/// drops every hit however well the engine ranked it.
pub(crate) async fn ranked_recall<P: MemoryRecall + ?Sized>(
    guard: &P,
    namespace: &str,
    query: &str,
    limit: usize,
) -> Result<RankedRecall, crate::memory::api::error::MemoryError> {
    log::debug!(
        "[memory:fallback] ranked recall via MemoryRecall::recall namespace={namespace} limit={limit}"
    );
    let opts = OwnedRecallOpts {
        namespace: Some(namespace.to_string()),
        ..Default::default()
    };
    let entries = guard.recall(query, limit, &opts, None).await?;
    let scored = entries.iter().any(|entry| entry.score.is_some());
    Ok(RankedRecall {
        hits: entries
            .into_iter()
            .take(limit)
            .map(|entry| entry_to_hit(entry, namespace))
            .collect(),
        scored,
    })
}

/// [`ranked_recall`]'s hits alone, with the error as text, for the RPCs that
/// list them.
pub(crate) async fn recall_hits<P: MemoryRecall + ?Sized>(
    guard: &P,
    namespace: &str,
    query: &str,
    limit: usize,
) -> Result<Vec<NamespaceMemoryHit>, String> {
    ranked_recall(guard, namespace, query, limit)
        .await
        .map(|ranked| ranked.hits)
        .map_err(|error| error.to_string())
}

/// Read one export record back as an entry (the mandatory driver's payload
/// shape: `key`, `content`, `category`, `session_id`, `timestamp`).
fn record_to_entry(
    record: crate::memory::api::provider::types::ExportRecord,
) -> Option<MemoryEntry> {
    let payload = &record.payload;
    let key = payload.get("key")?.as_str()?.to_string();
    let content = payload.get("content")?.as_str()?.to_string();
    let category = payload
        .get("category")
        .and_then(Value::as_str)
        .and_then(|c| c.parse().ok())
        .unwrap_or(crate::memory::api::types::MemoryCategory::Core);
    Some(MemoryEntry {
        id: record.id,
        key,
        content,
        namespace: record.namespace,
        category,
        timestamp: payload
            .get("timestamp")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        session_id: payload
            .get("session_id")
            .and_then(Value::as_str)
            .map(str::to_string),
        score: None,
        taint: record.taint,
    })
}

/// Most recently updated hits for `namespace`, newest first.
///
/// There is no query, so a ranked `recall` cannot answer this: the contract
/// says an empty query yields nothing. Served from at most
/// [`RECENT_MAX_PAGES`] `export_page` calls of `min(limit * 4, 200)` records
/// each, filtered to the namespace — never a full `list`. A page that holds no
/// record of the namespace triggers the second read; recency is therefore
/// best-effort within the engine's first pages.
pub(crate) async fn recent_hits(
    guard: &MemoryGuard,
    namespace: &str,
    limit: usize,
) -> Result<Vec<NamespaceMemoryHit>, String> {
    let page_size = limit.saturating_mul(4).clamp(1, RECENT_PAGE_CAP);
    log::debug!(
        "[memory:fallback] recency recall via bounded export namespace={namespace} limit={limit} page_size={page_size}"
    );
    let mut entries: Vec<MemoryEntry> = Vec::new();
    let mut cursor: Option<String> = None;
    for _ in 0..RECENT_MAX_PAGES {
        let page = guard
            .export_page(cursor.as_deref(), page_size)
            .await
            .map_err(|error| error.to_string())?;
        entries.extend(
            page.records
                .into_iter()
                .filter(|r| r.namespace.as_deref() == Some(namespace))
                .filter_map(record_to_entry),
        );
        cursor = page.next_cursor;
        if cursor.is_none() || !entries.is_empty() {
            break;
        }
    }
    entries.sort_by(|a, b| entry_epoch(b).total_cmp(&entry_epoch(a)));
    entries.truncate(limit);
    Ok(entries
        .into_iter()
        .map(|entry| entry_to_hit(entry, namespace))
        .collect())
}

fn document_json(entry: MemoryEntry, namespace: Option<&str>) -> Value {
    let ts = entry_epoch(&entry);
    json!({
        "documentId": entry.id,
        "namespace": entry.namespace.or_else(|| namespace.map(str::to_string)).unwrap_or_default(),
        "key": entry.key,
        "title": entry.key,
        "sourceType": "memory",
        "priority": "medium",
        "createdAt": ts,
        "updatedAt": ts,
    })
}

/// Document list in the `{"documents": [...], "truncated": bool}` shape
/// `list_documents` returns, built from `MemoryCore`.
///
/// With a namespace: one `list` call, capped at [`DOC_LIST_CAP`]. Without: one
/// `namespaces` call, then at most [`DOC_LIST_MAX_NAMESPACES`] `list` calls,
/// stopping at the cap. `truncated` says anything was left out.
pub(crate) async fn document_list(
    guard: &MemoryGuard,
    namespace: Option<&str>,
) -> Result<Value, String> {
    log::debug!("[memory:fallback] doc_list via MemoryCore::list namespace={namespace:?}");
    let mut documents: Vec<Value> = Vec::new();
    let mut truncated = false;
    let scopes: Vec<String> = match namespace {
        Some(ns) => vec![ns.to_string()],
        None => {
            let all = namespace_names(guard).await?;
            if all.len() > DOC_LIST_MAX_NAMESPACES {
                truncated = true;
            }
            all.into_iter().take(DOC_LIST_MAX_NAMESPACES).collect()
        }
    };
    for scope in scopes {
        let entries = guard
            .list(Some(&scope), None, None)
            .await
            .map_err(|error| error.to_string())?;
        for entry in entries {
            if documents.len() >= DOC_LIST_CAP {
                truncated = true;
                break;
            }
            documents.push(document_json(entry, Some(&scope)));
        }
        if documents.len() >= DOC_LIST_CAP {
            break;
        }
    }
    Ok(json!({ "documents": documents, "truncated": truncated }))
}

#[cfg(test)]
#[path = "fallback_tests.rs"]
mod tests;
