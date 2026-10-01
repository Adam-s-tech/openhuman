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

/// Seed enough records that a delayed hosted write keeps a migration running.
async fn seed_and_start_slow_migration(fx: &Fixture) -> String {
    for i in 0..3 {
        fx.put_doc(&format!("slow-{i}"), &format!("slow record {i}"))
            .await;
    }
    fx.hosted.delay_ms.store(700, Ordering::SeqCst);
    let v = fx
        .call(
            "openhuman.memory_engine_migrate",
            json!({ "to": { "driver": "tinyhumans" } }),
        )
        .await;
    result_of(&v, "engine_migrate")["job_id"]
        .as_str()
        .expect("job_id")
        .to_string()
}

#[test]
fn cancel_stops_a_running_migration_and_keeps_the_active_engine() {
    run_on_big_stack("engine-migrate-cancel", || async {
        let fx = Fixture::new().await;
        let job_id = seed_and_start_slow_migration(&fx).await;
        tokio::time::sleep(Duration::from_millis(150)).await;

        let v = fx
            .call(
                "openhuman.memory_engine_migrate_cancel",
                json!({ "job_id": job_id }),
            )
            .await;
        assert_eq!(result_of(&v, "cancel")["cancelled"], true);
        let status = fx.wait_job(&job_id).await;
        assert_eq!(status["state"], "cancelled", "{status}");
        assert_eq!(
            fx.state().await["driver"],
            "tinymemory",
            "a cancelled job must not switch"
        );
        fx.hosted.delay_ms.store(0, Ordering::SeqCst);
    });
}

#[test]
fn a_config_edited_during_the_copy_survives_the_switch() {
    run_on_big_stack("engine-migrate-stale-config", || async {
        let fx = Fixture::new().await;
        let job_id = seed_and_start_slow_migration(&fx).await;

        // Edit an unrelated setting on disk while the copy runs. The commit must
        // reload the config and patch only `[subsystems.memory]`, not write back
        // the snapshot taken when the RPC started.
        let path = shared_config_path();
        let mut toml = std::fs::read_to_string(&path).unwrap();
        toml = toml.replace(
            "default_model = \"e2e-mock-model\"",
            "default_model = \"edited-during-copy\"",
        );
        std::fs::write(&path, &toml).unwrap();

        let status = fx.wait_job(&job_id).await;
        assert_eq!(status["state"], "done", "{status}");
        let after = std::fs::read_to_string(&path).unwrap();
        assert!(
            after.contains("edited-during-copy"),
            "the commit clobbered a concurrent edit:\n{after}"
        );
        assert!(after.contains("driver = \"tinyhumans\""), "{after}");
        fx.hosted.delay_ms.store(0, Ordering::SeqCst);
        fx.call(
            "openhuman.memory_engine_set",
            json!({ "driver": "tinymemory" }),
        )
        .await;
    });
}
