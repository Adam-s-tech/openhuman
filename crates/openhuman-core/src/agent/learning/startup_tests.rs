use super::*;
use crate::agent::learning::candidate::Buffer;
use crate::agent::learning::extract::signature::{
    parse_signature, register_email_signature_subscriber_on,
};
use crate::core::events::DomainEvent;
use crate::memory::guard::test_support::RecordingProvider;
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;

/// A fresh temp workspace for the ready-arm test.
///
/// This used to build a real `MemoryClient` and hand it to
/// `register_with_client`, which never used it for anything but its
/// presence. Readiness is a `bool` now (#5560), so the fixture is just the
/// directory — but the host seams still have to be installed, because
/// `memory_is_bindable` and the facet cache below both resolve a driver and
/// an unwired embedding host fails loudly by design. `install_for_tests` is
/// `Once`-guarded, so calling it here is free when another test already has.
fn test_workspace() -> TempDir {
    TempDir::new().expect("tempdir")
}

/// A body whose trailing lines form a clear email signature — yields several
/// Identity candidates (name/role/timezone/employer).
fn signature_body() -> String {
    "Hi, great to hear from you!\n\n\
     Thanks,\n\
     Alice Johnson\n\
     Senior Software Engineer\n\
     Acme Corp\n\
     San Francisco, CA\n\
     PST"
    .to_string()
}

fn email_doc(source_id: &str, body: &str) -> DomainEvent {
    DomainEvent::DocumentCanonicalized {
        source_id: source_id.to_string(),
        source_kind: "email".to_string(),
        chunks_written: 1,
        chunk_ids: vec![format!("{source_id}-c1")],
        canonicalized_at: 0.0,
        body_preview: Some(body.to_string()),
    }
}

/// Poll an isolated buffer until at least `expected` candidates appear,
/// then settle briefly so an accidental duplicate subscription surfaces.
async fn wait_for_candidates(buffer: &Buffer, expected: usize) -> usize {
    for _ in 0..50 {
        if buffer.len() >= expected {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    tokio::time::sleep(Duration::from_millis(20)).await;
    buffer.len()
}

#[tokio::test]
async fn register_with_memory_registers_both_handles_when_ready() {
    crate::core::bus::init().await.expect("bus init");
    let tmp = test_workspace();
    let (trigger, renderer) =
        register_with_memory(true, tmp.path(), &MemorySubsystemConfig::default());
    assert!(
        trigger.is_some(),
        "rebuild trigger must register when memory is available"
    );
    assert!(
        renderer.is_some(),
        "ProfileMdRenderer must register when memory is available"
    );
}

#[tokio::test]
async fn register_with_memory_skips_and_warns_when_memory_absent() {
    // No memory driver → both memory-dependent subscribers are skipped and
    // the (now loud) warn path is exercised. This is the else-arm the #5003
    // fix upgraded from a silent debug-level skip.
    let tmp = TempDir::new().expect("tempdir");
    let (trigger, renderer) =
        register_with_memory(false, tmp.path(), &MemorySubsystemConfig::default());
    assert!(trigger.is_none(), "no trigger without a memory driver");
    assert!(renderer.is_none(), "no renderer without a memory driver");
}

#[tokio::test]
async fn learning_subscriber_fires_with_no_channel_configured() {
    let bus = crate::core::bus_testing::isolated_bus().await;
    let buffer: &'static Buffer = Box::leak(Box::new(Buffer::new(16)));
    let handle_cell = OnceLock::new();
    register_email_signature_once(&handle_cell, || {
        Some(register_email_signature_subscriber_on(&bus, buffer))
    });

    let source_id = "gmail:5003-e2e";
    let body = signature_body();
    let expected = parse_signature(&body, source_id, source_id).len();
    assert!(
        expected > 0,
        "signature body must yield at least one identity candidate"
    );

    bus.publish(email_doc(source_id, &body));
    let got = wait_for_candidates(buffer, expected).await;
    assert_eq!(
        got, expected,
        "email-signature subscriber must push the parsed identity candidates \
         with no channel configured anywhere (#5003)"
    );
}

#[tokio::test]
async fn register_learning_subscribers_is_idempotent() {
    let bus = crate::core::bus_testing::isolated_bus().await;
    let buffer: &'static Buffer = Box::leak(Box::new(Buffer::new(16)));
    let handle_cell = OnceLock::new();
    register_email_signature_once(&handle_cell, || {
        Some(register_email_signature_subscriber_on(&bus, buffer))
    });
    register_email_signature_once(&handle_cell, || {
        Some(register_email_signature_subscriber_on(&bus, buffer))
    });

    let source_id = "gmail:5003-idem";
    let body = signature_body();
    let expected = parse_signature(&body, source_id, source_id).len();
    assert!(expected > 0);

    bus.publish(email_doc(source_id, &body));
    let got = wait_for_candidates(buffer, expected).await;
    assert_eq!(
        got, expected,
        "double registration must not double the pushed candidates (#5003 idempotency)"
    );
}

/// An engine the test binds itself, so nothing reaches for the module.
fn chosen_engine() -> MemorySubsystemConfig {
    MemorySubsystemConfig {
        driver: "tinyhumans".into(),
        ..MemorySubsystemConfig::default()
    }
}

fn reached_profile(provider: &RecordingProvider) -> bool {
    provider
        .calls()
        .iter()
        .any(|call| call.method == "profile.list_all_facets")
}

/// Learning registers at boot, before the workspace's context exists, so it
/// asks the binding for the config the boot path hands it: the engine the user
/// chose reads as ready, and memory turned off reads as off.
#[test]
fn memory_readiness_comes_from_the_workspaces_own_config() {
    let tmp = test_workspace();
    let chosen = chosen_engine();
    crate::memory::binding::install_for_test(
        tmp.path(),
        &chosen,
        Arc::new(RecordingProvider::new()),
    );
    assert!(memory_is_bindable(tmp.path(), &chosen));

    let off = MemorySubsystemConfig {
        driver: "null".into(),
        ..MemorySubsystemConfig::default()
    };
    assert!(!memory_is_bindable(tmp.path(), &off));
}

/// The rebuild loop and the PROFILE.md renderer keep their cache for the whole
/// process. It reads through the engine the workspace's config names, and an
/// engine bound for the workspace later, as a switch binds one, is the one its
/// next call reaches.
#[tokio::test]
async fn a_long_lived_facet_cache_reads_the_engine_the_workspace_has_now() {
    let tmp = test_workspace();
    let chosen = chosen_engine();
    let first = Arc::new(RecordingProvider::new());
    crate::memory::binding::install_for_test(tmp.path(), &chosen, first.clone());
    let cache = facet_cache_for(tmp.path(), &chosen);
    cache
        .list_all()
        .await
        .expect("facets through the chosen engine");
    assert!(reached_profile(&first));

    let second = Arc::new(RecordingProvider::new());
    crate::memory::binding::install_for_test(tmp.path(), &chosen, second.clone());
    cache
        .list_all()
        .await
        .expect("facets through the engine bound since");
    assert!(
        reached_profile(&second),
        "a cache built before the switch must reach the engine bound after it"
    );
}

/// With no context serving the workspace, a switch still reaches the cache:
/// it takes the binding the switch left, not the config it was built with,
/// which would bind the engine switched away from again.
#[tokio::test]
async fn a_long_lived_facet_cache_follows_a_switch_made_without_a_context() {
    let tmp = test_workspace();
    let before = MemorySubsystemConfig {
        driver: "null".into(),
        ..MemorySubsystemConfig::default()
    };
    let first = Arc::new(RecordingProvider::new());
    crate::memory::binding::install_for_test(tmp.path(), &before, first.clone());
    let cache = facet_cache_for(tmp.path(), &before);
    cache
        .list_all()
        .await
        .expect("facets through the engine bound at first");
    assert!(reached_profile(&first));

    let after = chosen_engine();
    let second = Arc::new(RecordingProvider::new());
    crate::memory::binding::install_for_test(tmp.path(), &after, second.clone());
    crate::memory::binding::rebind(tmp.path(), &before.driver, &after).expect("switch");
    cache
        .list_all()
        .await
        .expect("facets through the engine switched to");
    assert!(
        reached_profile(&second),
        "the cache must follow the switch, not rebind the config it was built with"
    );
}
