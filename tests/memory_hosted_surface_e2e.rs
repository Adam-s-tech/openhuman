//! The memory surface on the hosted engine (openhuman#6718), against the same
//! hosted-CortexDB double as `memory_engine_e2e.rs`.
//!
//! The four memory suites (`memory_roundtrip_e2e`, `memory_sources_e2e`,
//! `memory_graph_roundtrip_e2e`, `memory_tree_health_e2e`) run on the local
//! module, whose engine serves every family. The hosted engine serves the
//! mandatory families plus ingestion and answers; this suite holds it to what
//! that means for a user: what it serves works, what it does not serve answers
//! a clean refusal rather than failing some other way, auto-recall reaches the
//! notes a user saved, and a refused lookup says why.
//!
//! ```text
//! RUST_MIN_STACK=67108864 cargo test -p openhuman-cli \
//!   --features "$(bash scripts/ci/product-features.sh)" \
//!   --test memory_hosted_surface_e2e
//! ```

#[path = "support/memory_engine_fixture.rs"]
mod fixture;

use std::sync::atomic::Ordering;

use fixture::*;
use openhuman_core::memory::api::types::{MemoryCategory, MemoryTaint};
use openhuman_core::memory::auto_recall::{AutoRecall, AUTO_RECALL_NOTES_NAMESPACE};
use serde_json::json;

/// A question the auto-recall gate opens for. The double's recall matches by
/// substring, so the note that should answer it quotes it.
const QUESTION: &str = "who is my idol and why?";

async fn bind_hosted(
    fx: &Fixture,
) -> std::sync::Arc<openhuman_core::memory::binding::MemoryBinding> {
    let v = fx
        .call(
            "openhuman.memory_engine_set",
            json!({ "driver": "tinyhumans" }),
        )
        .await;
    assert_eq!(
        result_of(&v, "engine_set tinyhumans")["driver"],
        "tinyhumans"
    );
    let config = openhuman_core::config::load_config_with_timeout()
        .await
        .unwrap();
    let binding = openhuman_core::memory::binding::for_config(&config).unwrap();
    assert_eq!(binding.driver_id(), "tinyhumans");
    binding
}

async fn unbind(fx: &Fixture) {
    fx.hosted.force_status.store(0, Ordering::SeqCst);
    fx.call(
        "openhuman.memory_engine_set",
        json!({ "driver": "tinymemory" }),
    )
    .await;
}

#[test]
fn auto_recall_reaches_a_saved_note_on_the_hosted_engine() {
    run_on_big_stack("hosted-auto-recall", || async {
        let fx = Fixture::new().await;
        let binding = bind_hosted(&fx).await;
        binding
            .provider()
            .store(
                AUTO_RECALL_NOTES_NAMESPACE,
                "idol",
                &format!("{QUESTION} The user's idol is Virat Kohli, for his discipline."),
                MemoryCategory::Core,
                None,
                MemoryTaint::Internal,
            )
            .await
            .expect("store the note");

        // Hosted recall carries no score: the note reads 0.0, which the scored
        // floor would drop. The lane keeps the engine's order instead.
        let lane = AutoRecall::from_guard(binding.guard());
        assert!(lane.enabled());
        let block = lane.block_for(QUESTION).await.expect("a block");
        assert!(block.contains("Virat Kohli"), "{block}");

        // Out of credits: the block says so instead of reading as "nothing
        // stored".
        fx.hosted.force_status.store(402, Ordering::SeqCst);
        let block = lane.block_for(QUESTION).await.expect("a refusal block");
        assert!(block.contains("out of credits"), "{block}");
        assert!(!block.contains("Virat Kohli"), "{block}");

        unbind(&fx).await;
    });
}

#[test]
fn a_refused_credential_and_an_outage_have_their_own_names() {
    run_on_big_stack("hosted-vocabulary", || async {
        let fx = Fixture::new().await;
        bind_hosted(&fx).await;

        // A 403 is a valid credential refused (an API key without the memory
        // scope). It must not read SESSION_EXPIRED, which signs the user out.
        fx.hosted.force_status.store(403, Ordering::SeqCst);
        let v = fx.call("openhuman.memory_list_namespaces", json!({})).await;
        let message = error_message(&v, "list_namespaces on 403");
        assert!(message.starts_with("MEMORY_FORBIDDEN:"), "{message}");
        assert!(!message.contains("SESSION_EXPIRED"), "{message}");

        // A 503 that outlasts the retries is the engine being unavailable.
        fx.hosted.force_status.store(503, Ordering::SeqCst);
        let v = fx.call("openhuman.memory_list_namespaces", json!({})).await;
        let message = error_message(&v, "list_namespaces on 503");
        assert!(message.starts_with("MEMORY_UNREACHABLE:"), "{message}");

        unbind(&fx).await;
    });
}

/// The handler answers from the host log when the driver keeps no run log
/// (`SourceSync` is absent), for a driver that serves the `Sources` sink
/// without owning pipelines. This fixture builds no `CoreContext`, so the
/// registry's capability gate stays open and the handler is reachable here.
/// In the app, an engine without `Sources` (hosted among them) has no
/// `memory_sources.*` methods at all, and Brain gates the panel on that.
#[test]
fn sync_history_answers_from_the_host_log_on_an_engine_without_one() {
    run_on_big_stack("hosted-sync-history", || async {
        let fx = Fixture::new().await;
        bind_hosted(&fx).await;
        let v = fx
            .call("openhuman.memory_sources_sync_audit_log", json!({}))
            .await;
        let history = result_of(&v, "sync_audit_log on hosted");
        assert!(history["entries"].is_array(), "{history}");
        unbind(&fx).await;
    });
}

/// What the hosted engine does not serve answers cleanly: the RPCs that need
/// a missing family name it, and the tree health reads degrade to an empty or
/// unhealthy answer rather than an error.
#[test]
fn what_the_hosted_engine_does_not_serve_is_refused_cleanly() {
    run_on_big_stack("hosted-refusals", || async {
        let fx = Fixture::new().await;
        bind_hosted(&fx).await;
        let refused = [
            // memory_graph_roundtrip_e2e
            (
                "openhuman.memory_graph_upsert",
                json!({ "namespace": NS, "subject": "a", "predicate": "knows", "object": "b" }),
            ),
            ("openhuman.memory_graph_query", json!({ "namespace": NS })),
            // memory_roundtrip_e2e
            (
                "openhuman.memory_doc_put",
                json!({
                    "namespace": NS, "key": "k", "title": "t", "content": "c",
                    "source_type": "doc", "priority": "medium", "tags": [],
                    "metadata": null, "category": "core"
                }),
            ),
        ];
        // Collected rather than asserted one by one, so a run names every
        // method that answers some other way.
        let mut offenders = Vec::new();
        for (method, params) in refused {
            let v = fx.call(method, params).await;
            let message = v
                .pointer("/error/message")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default();
            if !(message.contains("does not support the ") && message.contains(" family")) {
                offenders.push(format!("{method}: {v}"));
            }
        }
        assert!(
            offenders.is_empty(),
            "each must name the family the hosted engine lacks:\n{}",
            offenders.join("\n")
        );

        // memory_tree_health_e2e: the health reads answer from the host and
        // say what is missing, rather than failing.
        let v = fx
            .call("openhuman.memory_tree_list_sources", json!({}))
            .await;
        assert_eq!(result_of(&v, "tree list_sources on hosted"), &json!([]));
        let v = fx.call("openhuman.memory_tree_doctor", json!({})).await;
        let doctor = result_of(&v, "tree doctor on hosted");
        assert_eq!(doctor["healthy"], false, "{doctor}");
        assert!(
            doctor.to_string().contains("does not serve Maintenance"),
            "the doctor names the missing family: {doctor}"
        );
        let v = fx
            .call("openhuman.memory_tree_pipeline_status", json!({}))
            .await;
        assert_eq!(
            result_of(&v, "tree pipeline_status on hosted")["total_chunks"],
            0
        );

        unbind(&fx).await;
    });
}
