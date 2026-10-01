//! Fixtures for the guard's tests and for the domains that test through it.
//!
//! The recording fake itself is `tinymemory_conformance::RecordingProvider`: it
//! implements **all** the optional families and records what actually reached
//! it, so a test can assert both "the driver saw the value the guard rewrote"
//! and "the driver saw nothing at all". `NullMemoryProvider` cannot serve here —
//! it advertises only the mandatory three, so a guard built over it would have
//! no family decorators. What stays in the host is the part that names host
//! types: policies over a [`DriverClass`], and the guard wrapping.

#![cfg(test)]

use std::sync::Arc;

use crate::config::schema::MemoryHooksConfig;
use crate::core::subsystem::DriverClass;
use crate::memory::api::provider::MemoryProvider;
use crate::memory::api::types::{
    MemoryCategory, MemoryEntry, MemoryItemKind, MemoryTaint, NamespaceDocumentInput,
    NamespaceMemoryHit, RetrievalScoreBreakdown,
};
use crate::memory::guard::{HostGuardPolicy, MemoryGuard};

pub use tinymemory_conformance::RecordingProvider;

/// A [`HostGuardPolicy`] over an embedded driver with default budgets — the
/// shipped configuration.
pub fn embedded_policy() -> HostGuardPolicy {
    HostGuardPolicy::new(
        "recording",
        DriverClass::Embedded,
        MemoryHooksConfig::default(),
        super::policy::TRUSTED,
    )
}

/// A policy over an *external* driver. No such driver can bind today
/// (`binding::admit` refuses them), so this is the only way to reach the class
/// branches that land for real in M6.
pub fn external_policy(trust_state: &str) -> HostGuardPolicy {
    HostGuardPolicy::new(
        "supermemory",
        DriverClass::External,
        MemoryHooksConfig::default(),
        trust_state,
    )
}

/// A guard over a fresh recording provider, plus a handle on that provider.
pub fn guarded(policy: HostGuardPolicy) -> (Arc<RecordingProvider>, MemoryGuard) {
    guarded_with(RecordingProvider::new(), policy)
}

/// As [`guarded`], over a caller-configured provider.
pub fn guarded_with(
    provider: RecordingProvider,
    policy: HostGuardPolicy,
) -> (Arc<RecordingProvider>, MemoryGuard) {
    let provider = Arc::new(provider);
    let guard = MemoryGuard::new(
        Arc::clone(&provider) as Arc<dyn MemoryProvider>,
        Arc::new(policy),
    );
    (provider, guard)
}

/// A [`MemoryEntry`] fixture.
pub fn entry(content: &str) -> MemoryEntry {
    MemoryEntry {
        id: "id".into(),
        key: "key".into(),
        content: content.into(),
        namespace: Some("ns".into()),
        category: MemoryCategory::Core,
        timestamp: "2026-01-01T00:00:00Z".into(),
        session_id: None,
        score: None,
        taint: MemoryTaint::Internal,
    }
}

/// A [`NamespaceDocumentInput`] fixture.
pub fn document(content: &str, taint: MemoryTaint) -> NamespaceDocumentInput {
    NamespaceDocumentInput {
        namespace: "ns".into(),
        key: "k".into(),
        title: "t".into(),
        content: content.into(),
        source_type: "chat".into(),
        priority: "normal".into(),
        tags: vec![],
        metadata: serde_json::Value::Null,
        category: "core".into(),
        session_id: None,
        document_id: None,
        taint,
    }
}

/// A [`NamespaceMemoryHit`] with only the vector component set — the signal the
/// vector-floored recall paths (Lane B, the contradiction check) filter on.
pub fn namespace_hit(
    namespace: &str,
    key: &str,
    content: &str,
    vector_similarity: f64,
) -> NamespaceMemoryHit {
    NamespaceMemoryHit {
        id: format!("{namespace}/{key}"),
        kind: MemoryItemKind::Kv,
        namespace: namespace.into(),
        key: key.into(),
        title: None,
        content: content.into(),
        category: "core".into(),
        source_type: None,
        updated_at: 0.0,
        score: vector_similarity,
        score_breakdown: RetrievalScoreBreakdown {
            vector_similarity,
            ..Default::default()
        },
        document_id: None,
        chunk_id: None,
        supporting_relations: Vec::new(),
        taint: MemoryTaint::default(),
    }
}
