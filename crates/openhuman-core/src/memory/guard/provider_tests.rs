//! The host wiring of the guard: `HostGuardPolicy` fed by the ambient source
//! scope and the configured hook budgets, and step 7's bus event.
//!
//! The decorator's own behaviour (capability mirroring, taint raising, scope
//! narrowing, per-family enforcement) is tested with it, in `tinymemory-guard`.

use std::sync::Arc;

use crate::memory::api::provider::{MemoryCore, MemoryRecall};
use crate::memory::api::recall::OwnedRecallOpts;
use crate::memory::api::types::{MemoryCategory, MemoryTaint};

use crate::config::schema::MemoryHooksConfig;
use crate::core::bus::BUS;
use crate::core::events::DomainEvent;
use crate::core::subsystem::DriverClass;
use crate::memory::guard::policy::TRUSTED;
use crate::memory::guard::test_support::{
    embedded_policy, entry, external_policy, guarded, guarded_with, RecordingProvider,
};
use crate::memory::guard::HostGuardPolicy;
use crate::memory::source_scope::with_source_scope;

fn budgeted(recall_max_chars: usize, capture_max_chars: usize) -> HostGuardPolicy {
    HostGuardPolicy::new(
        "recording",
        DriverClass::Embedded,
        MemoryHooksConfig {
            recall_max_chars,
            capture_max_chars,
            ..MemoryHooksConfig::default()
        },
        TRUSTED,
    )
}

// ── Ambient scope and hook budgets ──────────────────────────────────────────

#[tokio::test]
async fn guard_stamps_taint_on_store_rather_than_trusting_the_caller() {
    let (driver, guard) = guarded(embedded_policy());
    with_source_scope(Some(vec!["slack:#eng".into()]), async {
        guard
            .store(
                "ns",
                "k",
                "hello",
                MemoryCategory::Core,
                None,
                MemoryTaint::Internal,
            )
            .await
            .expect("store");
    })
    .await;
    assert_eq!(driver.only_call().taint, Some(MemoryTaint::ExternalSync));
}

#[tokio::test]
async fn guard_truncates_stored_content_to_capture_max_chars() {
    let (driver, guard) = guarded(budgeted(1000, 5));
    guard
        .store(
            "ns",
            "k",
            "hello world",
            MemoryCategory::Core,
            None,
            MemoryTaint::Internal,
        )
        .await
        .expect("store");
    assert_eq!(driver.only_call().content.as_deref(), Some("hello"));
}

#[tokio::test]
async fn guard_truncates_recall_results_to_recall_max_chars() {
    let (_driver, guard) = guarded_with(
        RecordingProvider::new().with_recall_result(vec![
            entry("aaaa"),
            entry("bbbb"),
            entry("cccc"),
        ]),
        budgeted(6, 500),
    );
    let hits = guard
        .recall("q", 10, &OwnedRecallOpts::default(), None)
        .await
        .expect("recall");
    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0].content, "aaaa");
    assert_eq!(hits[1].content, "bb");
}

// ── Step 7 ──────────────────────────────────────────────────────────────────

struct DeniedRecorder {
    seen: std::sync::Mutex<Vec<(String, String, String)>>,
}

#[async_trait::async_trait]
impl tinybus::EventHandler<DomainEvent> for DeniedRecorder {
    fn name(&self) -> &str {
        "memory::guard::test_recorder"
    }

    async fn handle(&self, event: &DomainEvent) {
        if let DomainEvent::MemoryGuardDenied {
            driver_id,
            method,
            reason,
        } = event
        {
            self.seen.lock().expect("recorder mutex").push((
                driver_id.clone(),
                method.clone(),
                reason.clone(),
            ));
        }
    }
}

async fn record_denials() -> (Arc<DeniedRecorder>, tinybus::SubscriptionHandle) {
    crate::core::bus::init().await.expect("bus init");
    let recorder = Arc::new(DeniedRecorder {
        seen: std::sync::Mutex::new(Vec::new()),
    });
    let handle = BUS
        .subscribe(recorder.clone())
        .expect("the bus was just initialised");
    (recorder, handle)
}

async fn await_denial(recorder: &DeniedRecorder, driver_id: &str) -> (String, String, String) {
    for _ in 0..200 {
        if let Some(found) = recorder
            .seen
            .lock()
            .expect("recorder mutex")
            .iter()
            .find(|(id, _, _)| id == driver_id)
        {
            return found.clone();
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    panic!("no MemoryGuardDenied event for driver '{driver_id}' within 2s");
}

#[tokio::test]
async fn guard_publishes_memory_guard_denied_on_refusal() {
    let (recorder, _handle) = record_denials().await;
    let (driver, guard) = guarded(external_policy("untrusted"));
    let err = guard
        .store(
            "ns",
            "k",
            "hello",
            MemoryCategory::Core,
            None,
            MemoryTaint::Internal,
        )
        .await
        .expect_err("untrusted external driver must be refused");
    assert!(err.to_string().contains("memory guard: "));
    assert_eq!(driver.call_count(), 0, "the driver must never be reached");

    let (driver_id, method, reason) = await_denial(&recorder, "supermemory").await;
    assert_eq!(driver_id, "supermemory");
    assert_eq!(method, "core.store");
    assert!(!reason.contains("hello"), "must never carry content");
}

#[tokio::test]
async fn guard_publishes_nothing_on_the_success_path() {
    let (recorder, _handle) = record_denials().await;
    let (_driver, guard) = guarded(embedded_policy());
    guard
        .store(
            "ns",
            "k",
            "hello",
            MemoryCategory::Core,
            None,
            MemoryTaint::Internal,
        )
        .await
        .expect("store");
    guard
        .recall("q", 3, &OwnedRecallOpts::default(), None)
        .await
        .expect("recall");

    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    let seen = recorder.seen.lock().expect("recorder mutex");
    assert!(
        !seen
            .iter()
            .any(|(driver_id, _, _)| driver_id == "recording"),
        "a guarded read/write must not publish on success, saw: {seen:?}"
    );
}
