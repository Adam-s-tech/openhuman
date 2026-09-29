//! Memory-engine selection over JSON-RPC: migration, cancel, credits and config safety, against
//! a hosted-CortexDB double (see `support/memory_engine_fixture.rs`). List/get/set
//! are in `memory_engine_e2e.rs`.
//!
//! ```text
//! RUST_MIN_STACK=67108864 cargo test -p openhuman-cli \
//!   --features "$(bash scripts/ci/product-features.sh)" \
//!   --test memory_engine_migrate_e2e
//! ```

#[path = "support/memory_engine_fixture.rs"]
mod fixture;

use std::sync::atomic::Ordering;
use std::time::Duration;

use fixture::*;
use serde_json::json;

#[test]
fn engine_migrate_copies_the_module_into_the_hosted_engine_then_switches() {
    run_on_big_stack("engine-migrate", || async {
        let fx = Fixture::new().await;
        fx.put_doc("migrate-canary", "canary fact carried across engines")
            .await;

        let v = fx
            .call(
                "openhuman.memory_engine_migrate",
                json!({ "to": { "driver": "tinyhumans" } }),
            )
            .await;
        let job_id = result_of(&v, "engine_migrate")["job_id"]
            .as_str()
            .expect("job_id")
            .to_string();
        let status = fx.wait_job(&job_id).await;
        assert_eq!(status["state"], "done", "{status}");
        assert!(status["error"].is_null());
        assert!(status["copied"].as_u64().unwrap_or(0) >= 1, "{status}");

        assert!(
            fx.hosted
                .events
                .lock()
                .unwrap()
                .iter()
                .any(|e| e["content"]["text"]
                    .as_str()
                    .is_some_and(|t| t.contains("canary fact carried across engines"))),
            "the migrated record must exist in the hosted engine"
        );
        let state = fx.state().await;
        assert_eq!(
            state["driver"], "tinyhumans",
            "the switch commits after the copy: {state}"
        );

        let unknown = fx
            .call(
                "openhuman.memory_engine_migrate_status",
                json!({ "job_id": "nope" }),
            )
            .await;
        assert!(error_message(&unknown, "unknown job").contains("unknown"));

        // Migrating to the engine already in use is refused.
        let again = fx
            .call(
                "openhuman.memory_engine_migrate",
                json!({ "to": { "driver": "tinyhumans" } }),
            )
            .await;
        assert!(error_message(&again, "same engine").contains("already"));

        fx.call(
            "openhuman.memory_engine_set",
            json!({ "driver": "tinymemory" }),
        )
        .await;
    });
}

#[test]
fn insufficient_credits_fails_the_migration_and_keeps_the_active_engine() {
    run_on_big_stack("engine-migrate-402", || async {
        let fx = Fixture::new().await;
        fx.put_doc("credits-canary", "record that cannot be copied")
            .await;
        fx.hosted.force_status.store(402, Ordering::SeqCst);

        let v = fx
            .call(
                "openhuman.memory_engine_migrate",
                json!({ "to": { "driver": "tinyhumans" } }),
            )
            .await;
        let job_id = result_of(&v, "engine_migrate")["job_id"]
            .as_str()
            .unwrap()
            .to_string();
        let status = fx.wait_job(&job_id).await;
        assert_eq!(status["state"], "failed", "{status}");
        let message = status["error"].as_str().expect("failure reason");
        assert!(
            message.starts_with("INSUFFICIENT_CREDITS:"),
            "402 must map to INSUFFICIENT_CREDITS: {message}"
        );

        let state = fx.state().await;
        assert_eq!(
            state["driver"], "tinymemory",
            "a failed migration must not switch: {state}"
        );
        assert_eq!(state["class"], "module");
    });
}
