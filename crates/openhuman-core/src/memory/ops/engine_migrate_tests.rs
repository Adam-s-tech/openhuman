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
    let _guard = lock();
    clear();
}
