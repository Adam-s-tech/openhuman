use super::*;

/// The job map is process-global; serialise the cases that assert on it.
static JOBS_LOCK: Mutex<()> = Mutex::new(());

fn lock() -> std::sync::MutexGuard<'static, ()> {
    JOBS_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn clear() {
    jobs().clear();
}

#[test]
fn a_second_job_is_refused_while_one_is_running() {
    let _guard = lock();
    clear();
    let first = start_job().expect("first job starts");
    let refused = start_job().expect_err("only one migration at a time");
    assert!(refused.contains("already running"), "{refused}");
    update_job(&first, |j| j.state = "done".into());
    assert!(start_job().is_ok(), "a finished job frees the slot");
    clear();
}

#[test]
fn a_new_job_reports_running_with_no_progress() {
    let _guard = lock();
    clear();
    let id = start_job().unwrap();
    let job = jobs().get(&id).cloned().unwrap();
    assert_eq!(job.state, "running");
    assert_eq!(job.copied, 0);
    assert_eq!(job.total, None);
    assert_eq!(job.error, None);
    clear();
}

#[test]
fn fail_job_records_the_reason_and_stops_the_job() {
    let _guard = lock();
    clear();
    let id = start_job().unwrap();
    fail_job(&id, "INSUFFICIENT_CREDITS: nope".into());
    let job = jobs().get(&id).cloned().unwrap();
    assert_eq!(job.state, "failed");
    assert_eq!(job.error.as_deref(), Some("INSUFFICIENT_CREDITS: nope"));
    clear();
}

#[test]
fn finished_jobs_are_pruned_beyond_the_retention_bound() {
    let _guard = lock();
    clear();
    for _ in 0..(MAX_FINISHED_JOBS + 5) {
        let id = start_job().unwrap();
        update_job(&id, |j| j.state = "done".into());
    }
    assert!(jobs().len() <= MAX_FINISHED_JOBS, "{}", jobs().len());
    clear();
}

#[tokio::test]
async fn status_of_an_unknown_job_is_an_error() {
    let err = memory_engine_migrate_status(MigrateStatusParams {
        job_id: "does-not-exist".into(),
    })
    .await
    .expect_err("unknown job");
    assert!(err.contains("unknown"), "{err}");
}

#[tokio::test]
async fn status_reports_the_bare_shape() {
    let id = {
        let _guard = lock();
        clear();
        let id = start_job().unwrap();
        update_job(&id, |j| j.copied = 7);
        id
    };
    let outcome = memory_engine_migrate_status(MigrateStatusParams { job_id: id })
        .await
        .unwrap();
    let json = outcome.into_cli_compatible_json().unwrap();
    assert_eq!(json["state"], "running");
    assert_eq!(json["copied"], 7);
    assert!(json["total"].is_null());
    assert!(json["error"].is_null());
    assert!(json["step"].is_null(), "no step has reported yet");
    assert_eq!(json["step_read"], 0);
    assert_eq!(json["steps"], serde_json::json!([]));
    let _guard = lock();
    clear();
}

// ── Job runner: fakes, cancel, timeout, panic, serialisation ───────────────

use super::super::engine::EngineTargetParams;
use super::super::engine_fakes_tests::ScriptedProvider;
use crate::memory::api::provider::types::ImportOutcome;
use crate::memory::api::provider::MemoryProvider;
use std::sync::atomic::{AtomicBool, Ordering};

fn flag() -> Arc<AtomicBool> {
    Arc::new(AtomicBool::new(false))
}

fn as_provider(p: &Arc<ScriptedProvider>) -> Arc<dyn MemoryProvider> {
    Arc::clone(p) as Arc<dyn MemoryProvider>
}

async fn wait_terminal(id: &str) -> MigrateStatus {
    for _ in 0..200 {
        let status = jobs().get(id).cloned().unwrap();
        if status.state != "running" {
            return status;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    panic!("job did not finish");
}

#[cfg(feature = "memory-remote")]
#[allow(clippy::await_holding_lock)]
#[tokio::test]
async fn a_failed_import_fails_the_job_and_the_engine_does_not_switch() {
    let _guard = lock();
    clear();
    let source = ScriptedProvider::spread(5, 1);
    let target = ScriptedProvider::new(vec![]).with_import(ImportOutcome {
        failed: 1,
        errors: vec!["record rejected".into()],
        ..Default::default()
    });
    let id = start_job().unwrap();
    let switched = flag();
    let seen = Arc::clone(&switched);
    run_job(
        id.clone(),
        as_provider(&source),
        as_provider(&target),
        CopyChoices::default(),
        Duration::from_secs(30),
        move || async move {
            seen.store(true, Ordering::SeqCst);
            Ok(())
        },
    )
    .await;
    let status = jobs().get(&id).cloned().unwrap();
    assert_eq!(status.state, "failed", "{status:?}");
    assert!(status.error.unwrap().contains("could not be copied"));
    assert!(
        !switched.load(Ordering::SeqCst),
        "the engine must not switch"
    );
    clear();
}

#[cfg(feature = "memory-remote")]
#[allow(clippy::await_holding_lock)]
#[tokio::test]
async fn a_clean_copy_commits_the_switch_and_records_the_writes_note() {
    let _guard = lock();
    clear();
    let source = ScriptedProvider::spread(5, 1);
    let target = ScriptedProvider::new(vec![]);
    let id = start_job().unwrap();
    let switched = flag();
    let seen = Arc::clone(&switched);
    run_job(
        id.clone(),
        as_provider(&source),
        as_provider(&target),
        CopyChoices::default(),
        Duration::from_secs(30),
        move || async move {
            seen.store(true, Ordering::SeqCst);
            Ok(())
        },
    )
    .await;
    let status = jobs().get(&id).cloned().unwrap();
    assert_eq!(status.state, "done", "{status:?}");
    assert_eq!(status.copied, 5);
    assert!(
        status.note.is_some(),
        "the writes-during-copy caveat is reported"
    );
    assert!(switched.load(Ordering::SeqCst));
    assert_eq!(target.imported.lock().unwrap().len(), 5);
    clear();
}

#[cfg(feature = "memory-remote")]
#[allow(clippy::await_holding_lock)]
#[tokio::test]
async fn a_failing_commit_fails_the_job() {
    let _guard = lock();
    clear();
    let source = ScriptedProvider::spread(2, 1);
    let target = ScriptedProvider::new(vec![]);
    let id = start_job().unwrap();
    run_job(
        id.clone(),
        as_provider(&source),
        as_provider(&target),
        CopyChoices::default(),
        Duration::from_secs(30),
        || async { Err("the switch was saved but could not be applied".to_string()) },
    )
    .await;
    assert_eq!(jobs().get(&id).cloned().unwrap().state, "failed");
    clear();
}

#[cfg(feature = "memory-remote")]
#[allow(clippy::await_holding_lock)]
#[tokio::test]
async fn cancel_stops_the_copy_without_switching() {
    let _guard = lock();
    clear();
    let source = ScriptedProvider::spread(5, 1).hanging();
    let target = ScriptedProvider::new(vec![]);
    let id = start_job().unwrap();
    let switched = flag();
    let seen = Arc::clone(&switched);
    spawn_job(
        id.clone(),
        as_provider(&source),
        as_provider(&target),
        CopyChoices::default(),
        Duration::from_secs(30),
        move || async move {
            seen.store(true, Ordering::SeqCst);
            Ok(())
        },
    );
    tokio::time::sleep(Duration::from_millis(50)).await;
    let outcome = memory_engine_migrate_cancel(MigrateCancelParams { job_id: id.clone() })
        .await
        .unwrap();
    assert!(outcome.value.cancelled);
    let status = wait_terminal(&id).await;
    assert_eq!(status.state, "cancelled");
    assert!(!switched.load(Ordering::SeqCst));
    // A finished job cannot be cancelled again.
    let again = memory_engine_migrate_cancel(MigrateCancelParams { job_id: id })
        .await
        .unwrap();
    assert!(!again.value.cancelled);
    clear();
}

#[tokio::test]
async fn cancelling_an_unknown_job_is_an_error() {
    let err = memory_engine_migrate_cancel(MigrateCancelParams {
        job_id: "nope".into(),
    })
    .await
    .unwrap_err();
    assert!(err.contains("unknown"));
}

#[cfg(feature = "memory-remote")]
#[allow(clippy::await_holding_lock)]
#[tokio::test]
async fn the_overall_timeout_fails_the_job() {
    let _guard = lock();
    clear();
    let source = ScriptedProvider::spread(5, 1).hanging();
    let target = ScriptedProvider::new(vec![]);
    let id = start_job().unwrap();
    spawn_job(
        id.clone(),
        as_provider(&source),
        as_provider(&target),
        CopyChoices::default(),
        Duration::from_millis(50),
        || async { Ok(()) },
    );
    let status = wait_terminal(&id).await;
    assert_eq!(status.state, "failed");
    assert!(status.error.unwrap().contains("timed out"));
    clear();
}

#[cfg(feature = "memory-remote")]
#[allow(clippy::await_holding_lock)]
#[tokio::test]
async fn a_panicking_task_marks_the_job_failed() {
    let _guard = lock();
    clear();
    let source = ScriptedProvider::spread(5, 1).panicking();
    let target = ScriptedProvider::new(vec![]);
    let id = start_job().unwrap();
    spawn_job(
        id.clone(),
        as_provider(&source),
        as_provider(&target),
        CopyChoices::default(),
        Duration::from_secs(30),
        || async { Ok(()) },
    );
    let status = wait_terminal(&id).await;
    assert_eq!(status.state, "failed");
    assert!(status.error.unwrap().contains("panicked"));
    clear();
}

#[allow(clippy::await_holding_lock)]
#[tokio::test]
async fn engine_set_is_refused_while_a_migration_runs() {
    let _guard = lock();
    clear();
    let id = start_job().unwrap();
    let err = super::super::engine::memory_engine_set(EngineTargetParams {
        driver: "tinymemory".into(),
        endpoint: None,
        deployment: None,
        api_key: None,
    })
    .await
    .unwrap_err();
    assert!(err.contains("migration is running"), "{err}");
    update_job(&id, |j| j.state = "done".into());
    clear();
}

#[tokio::test]
async fn switches_are_serialised_on_one_lock() {
    let held = super::super::engine::SWITCH_LOCK.lock().await;
    let attempt = super::super::engine::memory_engine_set(EngineTargetParams {
        driver: "tinymemory".into(),
        endpoint: None,
        deployment: None,
        api_key: None,
    });
    assert!(
        tokio::time::timeout(Duration::from_millis(150), attempt)
            .await
            .is_err(),
        "a second switch must wait for the first"
    );
    drop(held);
}

// ── What the copy carries, and what holds the switch back ───────────────────

fn step(name: &str, failed: usize) -> MigrateStepStatus {
    MigrateStepStatus {
        step: name.to_string(),
        skipped_because: None,
        read: 10,
        written: 10 - failed,
        unchanged: 0,
        failed,
        errors: vec![format!("{name} item refused")],
    }
}

#[test]
fn a_host_synced_target_leaves_reader_sources_out_of_the_replay() {
    let hosted = copy_choices(true, true);
    assert!(hosted.replay_content);
    assert_eq!(hosted.skip_source_prefixes, vec!["mem_src:".to_string()]);
    let local = copy_choices(false, false);
    assert!(!local.replay_content);
    assert!(local.skip_source_prefixes.is_empty());
}

#[test]
fn the_replay_defaults_on_for_callers_that_do_not_say() {
    let params: MigrateParams =
        serde_json::from_value(serde_json::json!({ "to": { "driver": "tinycortex" } }))
            .expect("params");
    assert!(params.replay_content);
    let params: MigrateParams = serde_json::from_value(
        serde_json::json!({ "to": { "driver": "tinycortex" }, "replay_content": false }),
    )
    .expect("params");
    assert!(!params.replay_content);
}

#[test]
fn only_re_sent_content_may_fail_without_holding_the_switch_back() {
    assert!(blocking_failure(&[step("content", 3)]).is_none());
    let steps = [
        step("documents", 0),
        step("episodic", 2),
        step("content", 1),
    ];
    assert_eq!(
        blocking_failure(&steps).map(|s| s.step.as_str()),
        Some("episodic")
    );
}

#[test]
fn the_note_names_content_the_new_engine_refused() {
    let clean = finished_note(&[step("content", 0)]);
    assert_eq!(clean, WRITES_DURING_COPY_NOTE);
    let refused = finished_note(&[step("documents", 0), step("content", 4)]);
    assert!(refused.starts_with(WRITES_DURING_COPY_NOTE));
    assert!(refused.contains("4 pieces of synced content"), "{refused}");
}
