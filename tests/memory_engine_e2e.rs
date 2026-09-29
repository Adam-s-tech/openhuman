//! Memory-engine selection over JSON-RPC: list, get, set and validation, against
//! a hosted-CortexDB double (see `support/memory_engine_fixture.rs`). Migration
//! is in `memory_engine_migrate_e2e.rs`.
//!
//! ```text
//! RUST_MIN_STACK=67108864 cargo test -p openhuman-cli \
//!   --features "$(bash scripts/ci/product-features.sh)" \
//!   --test memory_engine_e2e
//! ```

#[path = "support/memory_engine_fixture.rs"]
mod fixture;

use std::sync::atomic::Ordering;

use fixture::*;
use serde_json::json;

#[test]
fn engines_list_and_get_report_the_module_by_default() {
    run_on_big_stack("engines-list", || async {
        let fx = Fixture::new().await;
        let v = fx.call("openhuman.memory_engines_list", json!({})).await;
        let list = result_of(&v, "engines_list");
        let engines = list["engines"].as_array().expect("engines array");
        let ids: Vec<&str> = engines.iter().filter_map(|e| e["id"].as_str()).collect();

        assert_eq!(ids[0], "tinymemory", "module first: {ids:?}");
        assert_eq!(engines[0]["label"], "TinyCortex (local)");
        for expected in [
            "tinyhumans",
            "supermemory",
            "mem0",
            "cognee",
            "cortex",
            "agentmemory",
        ] {
            assert!(ids.contains(&expected), "{expected} missing from {ids:?}");
        }
        assert!(!ids.contains(&"null"), "null must not be offered: {ids:?}");
        assert!(
            !ids.contains(&"tinycortex"),
            "in-memory tinycortex must not be offered: {ids:?}"
        );
        assert_eq!(list["active"], "tinymemory");
        let hosted = engines.iter().find(|e| e["id"] == "tinyhumans").unwrap();
        assert_eq!(hosted["hosted"], true);
        assert!(hosted["capabilities"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c == "answer"));

        let state = fx.state().await;
        assert_eq!(state["driver"], "tinymemory");
        assert_eq!(state["class"], "module");
        assert_eq!(state["has_credential"], false);
        assert!(state["fell_back_from"].is_null());
        assert!(state["last_error"].is_null());
    });
}

#[test]
fn engine_set_binds_the_hosted_engine_without_a_restart_and_switches_back() {
    run_on_big_stack("engine-set", || async {
        let fx = Fixture::new().await;

        let v = fx
            .call(
                "openhuman.memory_engine_set",
                json!({ "driver": "tinyhumans" }),
            )
            .await;
        let state = result_of(&v, "engine_set tinyhumans");
        assert_eq!(state["driver"], "tinyhumans", "{state}");
        assert_eq!(state["class"], "external");
        assert_eq!(
            state["has_credential"], true,
            "the live api key is the credential"
        );
        assert_eq!(state["endpoint"], fx.origin.as_str());
        assert!(state["fell_back_from"].is_null(), "{state}");
        assert!(
            !state.to_string().contains(TEST_API_KEY),
            "a secret leaked: {state}"
        );

        // The switch persisted: a fresh `get` (new config load) sees it.
        assert_eq!(fx.state().await["driver"], "tinyhumans");
        let status = fx.call("openhuman.memory_provider_status", json!({})).await;
        assert_eq!(
            result_of(&status, "provider_status")["driver"],
            "tinyhumans"
        );

        // A write through the bound provider reaches the double under the
        // session bearer, and reads back through the normal recall RPC.
        let config = openhuman_core::config::load_config_with_timeout()
            .await
            .unwrap();
        let binding = openhuman_core::memory::binding::for_config(&config).unwrap();
        assert_eq!(binding.driver_id(), "tinyhumans");
        binding
            .provider()
            .store(
                NS,
                "engine-note",
                "the hosted engine stores this",
                openhuman_core::memory::api::types::MemoryCategory::Core,
                None,
                openhuman_core::memory::api::types::MemoryTaint::Internal,
            )
            .await
            .expect("store through the hosted engine");
        assert!(
            fx.hosted
                .events
                .lock()
                .unwrap()
                .iter()
                .any(|e| e["content"]["text"]
                    .as_str()
                    .is_some_and(|t| t.contains("the hosted engine stores this"))),
            "the write must land in the hosted double"
        );
        assert!(
            fx.hosted
                .bearers
                .lock()
                .unwrap()
                .iter()
                .all(|b| b == TEST_API_KEY),
            "every hosted call carries the live credential"
        );
        // Reads through the mandatory recall family reach the hosted engine.
        let recalled = binding
            .provider()
            .recall(
                "hosted engine",
                5,
                &openhuman_core::memory::api::recall::OwnedRecallOpts {
                    namespace: Some(NS.to_string()),
                    ..Default::default()
                },
                None,
            )
            .await
            .expect("recall through the hosted engine");
        assert!(
            recalled
                .iter()
                .any(|e| e.content.contains("the hosted engine stores this")),
            "recall must be served by the hosted engine: {recalled:?}"
        );
        // The core memory RPCs work on a remote engine through the mandatory
        // families (engine-neutral fallbacks), and return the stored data.
        let v = fx.call("openhuman.memory_list_namespaces", json!({})).await;
        let namespaces = result_of(&v, "list_namespaces on hosted");
        assert!(
            namespaces.to_string().contains(NS),
            "list_namespaces must report the hosted namespace: {namespaces}"
        );
        assert!(
            namespaces.pointer("/data").is_some(),
            "envelope data: {namespaces}"
        );

        let v = fx
            .call(
                "openhuman.memory_recall_memories",
                json!({ "namespace": NS, "limit": 5 }),
            )
            .await;
        let recalled = result_of(&v, "recall_memories on hosted");
        assert!(
            recalled
                .to_string()
                .contains("the hosted engine stores this"),
            "recall_memories must return the stored memory: {recalled}"
        );

        let v = fx
            .call(
                "openhuman.memory_recall_context",
                json!({ "namespace": NS, "limit": 5 }),
            )
            .await;
        let context = result_of(&v, "recall_context on hosted");
        assert!(
            context
                .to_string()
                .contains("the hosted engine stores this"),
            "recall_context must return the stored memory: {context}"
        );

        let v = fx
            .call(
                "openhuman.memory_query_namespace",
                json!({ "namespace": NS, "query": "hosted engine" }),
            )
            .await;
        let queried = result_of(&v, "query_namespace on hosted");
        assert!(
            queried
                .to_string()
                .contains("the hosted engine stores this"),
            "query_namespace must return the ranked hit: {queried}"
        );

        let v = fx
            .call("openhuman.memory_doc_list", json!({ "namespace": NS }))
            .await;
        let docs = result_of(&v, "doc_list on hosted");
        assert!(
            docs.to_string().contains("engine-note"),
            "doc_list must list the stored entry: {docs}"
        );

        // Graph stays capability-gated, with the stable recognisable error.
        let v = fx
            .call("openhuman.memory_graph_query", json!({ "namespace": NS }))
            .await;
        let message = error_message(&v, "graph_query on hosted");
        assert!(
            message.contains("does not support the graph family"),
            "graph must be gated with the stable error: {message}"
        );

        // An unsupported family degrades to a clean error, not a panic.
        let doc = fx
            .call(
                "openhuman.memory_doc_put",
                json!({
                    "namespace": NS, "key": "k", "title": "t", "content": "c",
                    "source_type": "doc", "priority": "medium", "tags": [],
                    "metadata": null, "category": "core"
                }),
            )
            .await;
        assert!(
            error_message(&doc, "doc_put on the hosted engine")
                .contains("memory driver does not support the documents family"),
            "doc_put must name the missing family: {doc}"
        );

        // And back to the module, live.
        let v = fx
            .call(
                "openhuman.memory_engine_set",
                json!({ "driver": "tinymemory" }),
            )
            .await;
        let state = result_of(&v, "engine_set tinymemory");
        assert_eq!(state["driver"], "tinymemory");
        assert_eq!(state["class"], "module");
    });
}

#[test]
fn engine_set_validates_and_never_stores_or_returns_a_key_in_config() {
    run_on_big_stack("engine-set-validate", || async {
        let fx = Fixture::new().await;

        for bad in ["not-an-engine", "null", ""] {
            let v = fx
                .call("openhuman.memory_engine_set", json!({ "driver": bad }))
                .await;
            let message = error_message(&v, &format!("engine_set {bad:?}"));
            assert!(!message.is_empty());
        }
        let v = fx
            .call(
                "openhuman.memory_engine_set",
                json!({ "driver": "supermemory", "endpoint": "ftp://nope" }),
            )
            .await;
        assert!(error_message(&v, "bad endpoint").contains("http(s)"));
        let v = fx
            .call(
                "openhuman.memory_engine_set",
                json!({ "driver": "mem0", "deployment": "moon" }),
            )
            .await;
        assert!(error_message(&v, "bad deployment").contains("deployment"));
        assert_eq!(
            fx.state().await["driver"],
            "tinymemory",
            "a rejected set changes nothing"
        );

        let secret = "sm_secret_key_that_must_never_leak";
        let v = fx
            .call(
                "openhuman.memory_engine_set",
                json!({
                    "driver": "supermemory",
                    "endpoint": "https://api.supermemory.ai",
                    "api_key": secret
                }),
            )
            .await;
        let state = result_of(&v, "engine_set supermemory");
        assert_eq!(state["driver"], "supermemory", "{state}");
        assert_eq!(state["class"], "external");
        assert_eq!(state["has_credential"], true);
        assert!(
            !state.to_string().contains(secret),
            "key echoed in response: {state}"
        );
        let toml = std::fs::read_to_string(shared_config_path()).unwrap();
        assert!(!toml.contains(secret), "key written to config.toml");
        assert!(
            toml.contains("keychain:memory-supermemory"),
            "config should carry the ref: {toml}"
        );
        assert!(toml.contains("trust_state = \"trusted\""), "{toml}");

        fx.call(
            "openhuman.memory_engine_set",
            json!({ "driver": "tinymemory" }),
        )
        .await;
    });
}

#[test]
fn hosted_failures_on_ordinary_memory_rpcs_use_the_engine_error_vocabulary() {
    run_on_big_stack("engine-classify", || async {
        let fx = Fixture::new().await;
        fx.call(
            "openhuman.memory_engine_set",
            json!({ "driver": "tinyhumans" }),
        )
        .await;

        fx.hosted.force_status.store(402, Ordering::SeqCst);
        let v = fx.call("openhuman.memory_list_namespaces", json!({})).await;
        let message = error_message(&v, "list_namespaces on 402");
        assert!(
            message.starts_with("INSUFFICIENT_CREDITS:"),
            "a hosted 402 must read INSUFFICIENT_CREDITS: {message}"
        );

        fx.hosted.force_status.store(401, Ordering::SeqCst);
        let v = fx.call("openhuman.memory_list_namespaces", json!({})).await;
        let message = error_message(&v, "list_namespaces on 401");
        assert!(
            message.starts_with("SESSION_EXPIRED:"),
            "a hosted 401 must read SESSION_EXPIRED: {message}"
        );

        fx.hosted.force_status.store(0, Ordering::SeqCst);
        fx.call(
            "openhuman.memory_engine_set",
            json!({ "driver": "tinymemory" }),
        )
        .await;
    });
}
