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
async fn recent_hits_return_the_namespace_newest_first_within_the_limit() {
    use crate::memory::guard::in_memory::guard_over;
    use crate::memory::ops::engine_fakes_tests::ScriptedProvider;
    let provider = ScriptedProvider::spread(10, 2);
    let guard = guard_over(provider as std::sync::Arc<dyn MemoryProvider>);
    let hits = recent_hits(&guard, "ns0", 10).await.unwrap();
    assert_eq!(hits.len(), 5);
    assert!(hits.iter().all(|h| h.namespace == "ns0"));
    assert!(hits.windows(2).all(|w| w[0].updated_at >= w[1].updated_at));
    let one = recent_hits(&guard, "ns0", 1).await.unwrap();
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

// ── Bounded provider calls ──────────────────────────────────────────────────

use crate::memory::guard::in_memory::guard_over;
use crate::memory::ops::engine_fakes_tests::{entry, ScriptedProvider};
use std::sync::atomic::Ordering;

fn guarded(p: &std::sync::Arc<ScriptedProvider>) -> std::sync::Arc<MemoryGuard> {
    guard_over(std::sync::Arc::clone(p) as std::sync::Arc<dyn MemoryProvider>)
}

#[tokio::test]
async fn recent_hits_read_one_bounded_export_page_and_never_list() {
    let provider = ScriptedProvider::spread(1000, 1);
    let guard = guarded(&provider);
    let hits = recent_hits(&guard, "ns0", 5).await.unwrap();
    assert_eq!(hits.len(), 5);
    assert_eq!(
        provider.export_calls.load(Ordering::SeqCst),
        1,
        "one page suffices"
    );
    assert_eq!(
        provider.list_calls.load(Ordering::SeqCst),
        0,
        "never a full list"
    );
    assert_eq!(
        provider.max_export_limit.load(Ordering::SeqCst),
        20,
        "page size is limit * 4"
    );
    // Newest first: the scripted timestamps grow with the index.
    assert!(hits[0].updated_at >= hits[4].updated_at);
}

#[tokio::test]
async fn recent_hits_page_size_is_capped_at_200() {
    let provider = ScriptedProvider::spread(1000, 1);
    let guard = guarded(&provider);
    recent_hits(&guard, "ns0", 500).await.unwrap();
    assert_eq!(
        provider.max_export_limit.load(Ordering::SeqCst),
        RECENT_PAGE_CAP
    );
}

#[tokio::test]
async fn recent_hits_read_at_most_two_pages_when_the_namespace_is_absent() {
    // Everything lives in another namespace: the reader must give up after two
    // pages instead of walking the whole store.
    let provider = ScriptedProvider::new(
        (0..500)
            .map(|i| entry("elsewhere", &format!("k{i}"), "x", "2026-01-01T00:00:00Z"))
            .collect(),
    );
    let guard = guarded(&provider);
    let hits = recent_hits(&guard, "wanted", 5).await.unwrap();
    assert!(hits.is_empty());
    assert_eq!(
        provider.export_calls.load(Ordering::SeqCst),
        RECENT_MAX_PAGES
    );
    assert_eq!(provider.list_calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn document_list_without_a_namespace_scans_a_bounded_number_of_namespaces() {
    let provider = ScriptedProvider::spread(3000, 40);
    let guard = guarded(&provider);
    let raw = document_list(&guard, None).await.unwrap();
    assert_eq!(provider.namespaces_calls.load(Ordering::SeqCst), 1);
    assert!(
        provider.list_calls.load(Ordering::SeqCst) <= DOC_LIST_MAX_NAMESPACES,
        "at most {DOC_LIST_MAX_NAMESPACES} namespaces are listed"
    );
    assert!(raw["documents"].as_array().unwrap().len() <= DOC_LIST_CAP);
    assert_eq!(raw["truncated"], true, "40 namespaces cannot all be listed");
}

#[tokio::test]
async fn document_list_for_one_namespace_is_one_call_capped_at_200() {
    let provider = ScriptedProvider::spread(600, 2);
    let guard = guarded(&provider);
    let raw = document_list(&guard, Some("ns0")).await.unwrap();
    assert_eq!(provider.list_calls.load(Ordering::SeqCst), 1);
    assert_eq!(raw["documents"].as_array().unwrap().len(), DOC_LIST_CAP);
    assert_eq!(raw["truncated"], true);

    let small = ScriptedProvider::spread(10, 2);
    let raw = document_list(&guarded(&small), Some("ns1")).await.unwrap();
    assert_eq!(raw["documents"].as_array().unwrap().len(), 5);
    assert_eq!(raw["truncated"], false);
}

#[test]
fn hosted_errors_are_classified_only_for_external_engines() {
    let credits =
        "budget exceeded: [USER_INSUFFICIENT_CREDITS] memory API x (HTTP 402)".to_string();
    assert!(classify_for_engine(true, credits.clone()).starts_with("INSUFFICIENT_CREDITS:"));
    assert_eq!(classify_for_engine(false, credits.clone()), credits);

    let session = "unauthorized: [UNAUTHORIZED] memory API x (HTTP 401)".to_string();
    assert!(classify_for_engine(true, session).starts_with("SESSION_EXPIRED:"));

    let gate = unsupported_family("graph");
    assert_eq!(
        classify_for_engine(true, gate.clone()),
        gate,
        "the gate error is stable"
    );
    assert_eq!(classify_for_engine(true, "boom".into()), "boom");
}

#[test]
fn error_envelopes_carry_the_classified_message() {
    let outcome = crate::memory::ops::envelope::error_envelope::<()>(
        "memory.recall_failed",
        "budget exceeded: [USER_INSUFFICIENT_CREDITS] memory API x (HTTP 402)".to_string(),
    );
    let message = outcome.value.error.expect("error").message;
    assert!(message.starts_with("INSUFFICIENT_CREDITS:"), "{message}");
}
