//! The memory surface on the hosted engine (openhuman#6718), against the same
//! hosted-CortexDB double as `memory_engine_e2e.rs`.
//!
//! The four memory suites (`memory_roundtrip_e2e`, `memory_sources_e2e`,
//! `memory_graph_roundtrip_e2e`, `memory_tree_health_e2e`) run on the local
//! module, whose engine serves every family. The hosted engine serves the
//! mandatory families, ingestion and answers, and goals, tool rules, documents,
//! the source sink, maintenance, retrieval, ingest, profile, episodic memory,
//! scoring and a tree drawn from the server's understanding (tinymemory's
//! hosted-families spec); local sources are synced by the host through its
//! sink. This suite holds it to what that means for a user: what it serves
//! works, what it does not serve answers a clean refusal rather than failing
//! some other way, auto-recall reaches the notes a user saved, a refused lookup
//! says why, the Brain graph draws what the server understood, and a synced
//! folder reaches hosted memory.
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

/// The handler answers from the host log when the driver keeps no run log:
/// hosted memory serves the `Sources` sink without owning pipelines
/// (`SourceSync` is absent), so its `memory_sources.*` methods are registered
/// and Sync History reads what the host recorded.
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

/// The Brain graph on hosted memory is the server's understanding: a fact it
/// derived from the user's notes is a node labelled by what it says, under
/// the namespace it came from, named for what that holds.
#[test]
fn the_hosted_engine_draws_the_brain_graph_from_its_understanding() {
    run_on_big_stack("hosted-graph", || async {
        let fx = Fixture::new().await;
        bind_hosted(&fx).await;
        fx.hosted.layers.lock().unwrap().insert(
            ("facts".to_string(), "tm:global".to_string()),
            vec![json!({
                "id": "fact_tea",
                "scope": "tm:global",
                "subject": { "type": "entity", "id": "ent_user", "name": "User" },
                "predicate": "prefers",
                "object": { "type": "literal", "datatype": "string", "value": "oolong tea" },
                "supports": [],
                "confidence": 0.9,
                "valid_from": "2026-09-01T00:00:00Z",
                "recorded_from": "2026-09-01T00:00:00Z",
            })],
        );
        let v = fx
            .call(
                "openhuman.memory_tree_graph_export",
                json!({ "mode": "tree" }),
            )
            .await;
        let graph = result_of(&v, "graph export on hosted");
        let labels: Vec<&str> = graph["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .filter_map(|node| node["label"].as_str())
            .collect();
        assert!(labels.contains(&"User prefers oolong tea"), "{graph}");
        assert!(
            labels.contains(&"Memory"),
            "the global namespace reads as what it holds: {graph}"
        );
        unbind(&fx).await;
    });
}

/// A folder source on hosted memory is read by the host and sent through the
/// engine's sink, and the run lands in Sync History like any other.
#[test]
fn a_local_folder_syncs_into_hosted_memory() {
    run_on_big_stack("hosted-folder-sync", || async {
        let fx = Fixture::new().await;
        bind_hosted(&fx).await;
        let folder = fx._tmp.path().join("notes");
        std::fs::create_dir_all(&folder).expect("folder");
        std::fs::write(folder.join("tea.md"), "# Tea\n\nOolong, always.").expect("note");
        let v = fx
            .call(
                "openhuman.memory_sources_add",
                json!({
                    "kind": "folder",
                    "label": "Notes",
                    "enabled": true,
                    "path": folder.to_string_lossy(),
                }),
            )
            .await;
        let source_id = result_of(&v, "add a folder source")["source"]["id"]
            .as_str()
            .expect("a source id")
            .to_string();
        let v = fx
            .call(
                "openhuman.memory_sources_sync",
                json!({ "source_id": source_id }),
            )
            .await;
        result_of(&v, "sync a folder on hosted");
        let synced = fx.hosted.events.lock().unwrap().iter().any(|event| {
            event["scope"] == "tm:sources/tm:documents"
                && event["content"]["text"]
                    .as_str()
                    .is_some_and(|text| text.contains("Oolong, always."))
        });
        assert!(synced, "the note reaches the hosted documents namespace");
        let v = fx
            .call("openhuman.memory_sources_sync_audit_log", json!({}))
            .await;
        let history = result_of(&v, "sync history after a hosted folder sync");
        assert!(
            history["entries"].to_string().contains(&source_id),
            "{history}"
        );
        unbind(&fx).await;
    });
}

/// Adds `folder` as a folder source and answers its id.
async fn add_folder(fx: &Fixture, folder: &std::path::Path) -> String {
    let v = fx
        .call(
            "openhuman.memory_sources_add",
            json!({
                "kind": "folder",
                "label": "Notes",
                "enabled": true,
                "path": folder.to_string_lossy(),
            }),
        )
        .await;
    result_of(&v, "add a folder source")["source"]["id"]
        .as_str()
        .expect("a source id")
        .to_string()
}

/// Whether the hosted double holds an event whose text contains `needle`.
fn hosted_holds(fx: &Fixture, needle: &str) -> bool {
    fx.hosted.events.lock().unwrap().iter().any(|event| {
        event["content"]["text"]
            .as_str()
            .is_some_and(|text| text.contains(needle))
    })
}

/// A file deleted from a host-synced folder is forgotten from hosted memory
/// on the next sync, by the id the sink answered for it. A walk that finds
/// the folder empty forgets nothing and keeps the record, so a later walk
/// still can.
#[test]
fn a_file_deleted_from_a_synced_folder_is_forgotten_from_hosted_memory() {
    run_on_big_stack("hosted-folder-delete", || async {
        let fx = Fixture::new().await;
        bind_hosted(&fx).await;
        let folder = fx._tmp.path().join("deletes");
        std::fs::create_dir_all(&folder).expect("folder");
        std::fs::write(folder.join("tea.md"), "# Tea\n\nOolong, always.").expect("tea");
        std::fs::write(folder.join("coffee.md"), "# Coffee\n\nNever before noon.").expect("coffee");
        let source_id = add_folder(&fx, &folder).await;
        let sync = || {
            fx.call(
                "openhuman.memory_sources_sync",
                json!({ "source_id": source_id }),
            )
        };
        result_of(&sync().await, "first sync");
        assert!(hosted_holds(&fx, "Never before noon."), "coffee is synced");

        let tea = std::fs::read_to_string(folder.join("tea.md")).expect("read tea");
        std::fs::remove_file(folder.join("tea.md")).expect("hide tea");
        std::fs::remove_file(folder.join("coffee.md")).expect("delete coffee");
        result_of(&sync().await, "sync over an empty folder");
        assert!(
            hosted_holds(&fx, "Never before noon."),
            "an empty walk forgets nothing"
        );

        std::fs::write(folder.join("tea.md"), tea).expect("tea is back");
        result_of(&sync().await, "sync after the delete");
        assert!(
            !hosted_holds(&fx, "Never before noon."),
            "the deleted file is forgotten by its stored id"
        );
        assert!(
            hosted_holds(&fx, "Oolong, always."),
            "the file still there stays"
        );
        unbind(&fx).await;
    });
}

/// The first `documentId` anywhere in `value`.
fn document_id(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::Object(map) => map
            .get("documentId")
            .and_then(serde_json::Value::as_str)
            .map(str::to_string)
            .or_else(|| map.values().find_map(document_id)),
        serde_json::Value::Array(items) => items.iter().find_map(document_id),
        _ => None,
    }
}

/// The families hosted memory serves beyond the mandatory ones, through the
/// app's RPCs: documents and tool rules round-trip.
#[test]
fn the_hosted_engine_serves_documents_and_tool_rules() {
    run_on_big_stack("hosted-families", || async {
        let fx = Fixture::new().await;
        bind_hosted(&fx).await;

        let v = fx
            .call(
                "openhuman.memory_doc_put",
                json!({
                    "namespace": NS, "key": "tea", "title": "Tea",
                    "content": "The user drinks oolong.", "source_type": "doc",
                    "priority": "medium", "tags": [], "metadata": null,
                    "category": "core"
                }),
            )
            .await;
        result_of(&v, "doc_put on hosted");
        let v = fx
            .call("openhuman.memory_doc_list", json!({ "namespace": NS }))
            .await;
        let listed = result_of(&v, "doc_list on hosted").clone();
        assert!(listed.to_string().contains("Tea"), "{listed}");
        let id = document_id(&listed).unwrap_or_else(|| panic!("no document id: {listed}"));
        let v = fx
            .call(
                "openhuman.memory_doc_delete",
                json!({ "namespace": NS, "document_id": id }),
            )
            .await;
        let deleted = result_of(&v, "doc_delete on hosted");
        assert!(
            deleted.to_string().contains("\"deleted\":true"),
            "{deleted}"
        );

        let v = fx
            .call(
                "openhuman.memory_tool_rule_put",
                json!({ "tool_name": "shell", "rule": "quote every path" }),
            )
            .await;
        result_of(&v, "tool_rule_put on hosted");
        let v = fx
            .call(
                "openhuman.memory_tool_rule_list",
                json!({ "tool_name": "shell" }),
            )
            .await;
        let rules = result_of(&v, "tool_rule_list on hosted");
        assert!(rules.to_string().contains("quote every path"), "{rules}");

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
        // Hosted memory serves Maintenance: the doctor answers from the
        // service's health probe rather than naming a missing family.
        let v = fx.call("openhuman.memory_tree_doctor", json!({})).await;
        let doctor = result_of(&v, "tree doctor on hosted");
        assert_eq!(doctor["healthy"], true, "{doctor}");
        assert!(
            doctor.to_string().contains("\"service\""),
            "the doctor reports the hosted service stage: {doctor}"
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
