//! The fallbacks answer from a provider that implements only the mandatory
//! families (`InMemoryProvider`), behind a real guard.

use super::*;
use crate::memory::api::provider::MemoryProvider;
use crate::memory::api::types::{MemoryCategory, MemoryTaint};
use crate::memory::guard::in_memory::guarded_in_memory;

async fn seeded() -> std::sync::Arc<MemoryGuard> {
    let (provider, guard) = guarded_in_memory();
    assert!(
        provider.as_documents().is_none() && provider.as_retrieval().is_none(),
        "the double must expose only the mandatory families"
    );
    for (ns, key, content) in [
        ("notes", "a", "alpha rust note"),
        ("notes", "b", "beta python note"),
        ("work", "c", "gamma rust task"),
    ] {
        guard
            .store(
                ns,
                key,
                content,
                MemoryCategory::Core,
                None,
                MemoryTaint::Internal,
            )
            .await
            .unwrap();
    }
    guard
}

#[tokio::test]
async fn namespace_names_come_from_the_core_surface() {
    let guard = seeded().await;
    assert_eq!(
        namespace_names(&guard).await.unwrap(),
        vec!["notes".to_string(), "work".to_string()]
    );
}

#[tokio::test]
async fn recall_hits_use_mandatory_recall_scoped_to_the_namespace() {
    let guard = seeded().await;
    let hits = recall_hits(guard.as_ref(), "notes", "rust", 5)
        .await
        .unwrap();
    assert_eq!(hits.len(), 1, "namespace scoping must hold: {hits:?}");
    assert_eq!(hits[0].key, "a");
    assert_eq!(hits[0].namespace, "notes");
    assert_eq!(hits[0].content, "alpha rust note");
    assert_eq!(hits[0].source_type.as_deref(), Some("recall"));

    let limited = recall_hits(guard.as_ref(), "notes", "note", 1)
        .await
        .unwrap();
    assert_eq!(limited.len(), 1, "limit must be honoured");
}

#[tokio::test]
async fn recent_hits_list_the_namespace_newest_first_within_the_limit() {
    let guard = seeded().await;
    let hits = recent_hits(&guard, "notes", 10).await.unwrap();
    assert_eq!(hits.len(), 2);
    let one = recent_hits(&guard, "notes", 1).await.unwrap();
    assert_eq!(one.len(), 1);
    assert!(recent_hits(&guard, "missing", 5).await.unwrap().is_empty());
}

#[tokio::test]
async fn document_list_matches_the_summary_shape_the_rpc_parses() {
    let guard = seeded().await;
    let raw = document_list(&guard, Some("notes")).await.unwrap();
    let docs = crate::memory::ops::helpers::parse_memory_document_summaries(raw)
        .expect("fallback must parse as document summaries");
    assert_eq!(docs.len(), 2);
    assert!(docs.iter().all(|d| d.namespace == "notes"));
    assert!(docs.iter().any(|d| d.key == "a" && d.title == "a"));

    let all = document_list(&guard, None).await.unwrap();
    assert_eq!(all["documents"].as_array().unwrap().len(), 3);
}

#[test]
fn the_capability_gate_error_is_stable_and_recognisable() {
    let message = unsupported_family("graph");
    assert_eq!(message, "memory driver does not support the graph family");
    assert!(is_unsupported_family_error(&message));
    assert!(!is_unsupported_family_error("connection refused"));
}
