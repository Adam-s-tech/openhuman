//! `memory.engine_migrate` / `memory.engine_migrate_status` — copy everything
//! from the active engine into another, then switch to it.
//!
//! The copy is `tinymemory::migrate::copy`: engine-neutral, at-least-once and
//! non-destructive (the source keeps its data, and the target skips records it
//! recognises), so a failed or repeated migration is safe and the previous
//! engine stays usable. The switch is committed only after the copy finished
//! with no failed records; on any failure the active engine is untouched.
//!
//! Jobs live in an in-process map (one at a time). They do not survive a
//! restart: an interrupted migration is simply run again.
//!
//! - **Cancel** (`engine_migrate_cancel`): the copy future is raced against a
//!   cancel signal and dropped, which stops it between awaits (i.e. between
//!   pages). The previous engine is untouched.
//! - **Timeout**: the whole copy is bounded by `OPENHUMAN_MEMORY_MIGRATE_TIMEOUT_SECS`
//!   (default 2 hours). A panic in the task also fails the job.
//! - **Writes during the copy**: the export contract has no portable record
//!   timestamp, so no delta pass is attempted. A successful job carries a
//!   `note` saying records written while it ran may be missing from the new
//!   engine; the old engine still holds them and a second migration copies them
//!   (targets skip records they already have).

use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

use tokio::sync::Notify;

use serde::{Deserialize, Serialize};

use super::engine::{
    build_target_provider, classify_engine_error, classify_engine_message, commit_engine,
    prepare_target, EngineTargetParams, SWITCH_LOCK,
};
use crate::config::schema::Config;
use crate::core::runtime::context::CoreContext;
use crate::core::Outcome;
use crate::memory::binding::{self, MODULE_ID};

const LOG_PREFIX: &str = "[memory:engine-migrate]";
/// Finished jobs kept for status polling.
const MAX_FINISHED_JOBS: usize = 16;

/// Parameters of `memory.engine_migrate`.
#[derive(Clone, Deserialize)]
pub struct MigrateParams {
    pub to: EngineTargetParams,
}

/// Parameters of `memory.engine_migrate_cancel`.
#[derive(Debug, Clone, Deserialize)]
pub struct MigrateCancelParams {
    pub job_id: String,
}

/// Result of `memory.engine_migrate_cancel`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MigrateCancelled {
    /// Whether a running job was signalled. `false` when it had already
    /// finished (or is past the point of no return, committing the switch).
    pub cancelled: bool,
}

/// Default bound on one migration's copy phase.
const DEFAULT_TIMEOUT_SECS: u64 = 2 * 60 * 60;

fn migrate_timeout() -> Duration {
    let secs = std::env::var("OPENHUMAN_MEMORY_MIGRATE_TIMEOUT_SECS")
        .ok()
        .and_then(|v| v.trim().parse::<u64>().ok())
        .filter(|s| *s > 0)
        .unwrap_or(DEFAULT_TIMEOUT_SECS);
    Duration::from_secs(secs)
}

/// Parameters of `memory.engine_migrate_status`.
#[derive(Debug, Clone, Deserialize)]
pub struct MigrateStatusParams {
    pub job_id: String,
}

/// Result of `memory.engine_migrate`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MigrateStarted {
    pub job_id: String,
}

/// Result of `memory.engine_migrate_status`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MigrateStatus {
    /// `running` | `done` | `failed` | `cancelled`.
    pub state: String,
    /// Records read from the source and handed to the target so far.
    pub copied: usize,
    /// Total records when known; the export cursor does not report one.
    pub total: Option<usize>,
    pub error: Option<String>,
    /// A caveat on a finished job (see the module docs on writes during the
    /// copy); `None` otherwise.
    pub note: Option<String>,
}

/// Cancel signals of the jobs in [`JOBS`].
static CANCELS: OnceLock<Mutex<HashMap<String, Arc<Notify>>>> = OnceLock::new();

fn cancels() -> std::sync::MutexGuard<'static, HashMap<String, Arc<Notify>>> {
    CANCELS
        .get_or_init(Default::default)
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Whether any migration job is still running. `engine_set` refuses meanwhile.
pub(super) fn migration_running() -> bool {
    jobs().values().any(|j| j.state == "running")
}

static JOBS: OnceLock<Mutex<HashMap<String, MigrateStatus>>> = OnceLock::new();

fn jobs() -> std::sync::MutexGuard<'static, HashMap<String, MigrateStatus>> {
    JOBS.get_or_init(Default::default)
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn update_job(job_id: &str, f: impl FnOnce(&mut MigrateStatus)) {
    if let Some(job) = jobs().get_mut(job_id) {
        f(job);
    }
}

fn fail_job(job_id: &str, error: String) {
    log::warn!("{LOG_PREFIX} job={job_id} failed: {error}");
    update_job(job_id, |j| {
        j.state = "failed".to_string();
        j.error = Some(error);
    });
}

/// Register a new running job, refusing while another is running.
fn start_job() -> Result<String, String> {
    let mut map = jobs();
    if map.values().any(|j| j.state == "running") {
        return Err("a memory migration is already running".to_string());
    }
    // Only finished jobs can be here (a running one was refused above).
    while map.len() >= MAX_FINISHED_JOBS {
        let Some(oldest) = map.keys().next().cloned() else {
            break;
        };
        map.remove(&oldest);
        cancels().remove(&oldest);
    }
    let job_id = uuid::Uuid::new_v4().to_string();
    map.insert(
        job_id.clone(),
        MigrateStatus {
            state: "running".to_string(),
            copied: 0,
            total: None,
            error: None,
            note: None,
        },
    );
    cancels().insert(job_id.clone(), Arc::new(Notify::new()));
    Ok(job_id)
}

/// What a finished copy reports.
struct CopyReport {
    records: usize,
    imported: usize,
    skipped: usize,
    failed: usize,
    errors: Vec<String>,
}

#[cfg(feature = "memory-remote")]
async fn run_copy(
    source: &dyn crate::memory::api::provider::MemoryProvider,
    target: &dyn crate::memory::api::provider::MemoryProvider,
    on_progress: impl FnMut(usize),
) -> anyhow::Result<CopyReport> {
    let mut on_progress = on_progress;
    let report = tinymemory::migrate::copy(source, target, |p| on_progress(p.records)).await?;
    Ok(CopyReport {
        records: report.records,
        imported: report.imported,
        skipped: report.skipped,
        failed: report.failed,
        errors: report.errors,
    })
}

#[cfg(not(feature = "memory-remote"))]
async fn run_copy(
    _source: &dyn crate::memory::api::provider::MemoryProvider,
    _target: &dyn crate::memory::api::provider::MemoryProvider,
    _on_progress: impl FnMut(usize),
) -> anyhow::Result<CopyReport> {
    anyhow::bail!("memory engine migration is not compiled into this build")
}

/// How the copy phase ended.
enum CopyEnd {
    Finished(anyhow::Result<CopyReport>),
    Cancelled,
    TimedOut,
}

/// Run one job to a terminal state: copy (cancellable, bounded), then — only on
/// a clean copy — `commit` the engine switch. Never panics into the caller; a
/// failure at any point leaves the active engine unchanged.
async fn run_job<F, Fut>(
    job_id: String,
    source: Arc<dyn crate::memory::api::provider::MemoryProvider>,
    target: Arc<dyn crate::memory::api::provider::MemoryProvider>,
    timeout: Duration,
    commit: F,
) where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<(), String>>,
{
    let cancel = cancels().get(&job_id).cloned().unwrap_or_default();
    let progress_id = job_id.clone();
    let copy = run_copy(source.as_ref(), target.as_ref(), move |records| {
        update_job(&progress_id, |j| j.copied = records);
    });
    // Dropping the copy future on cancel/timeout stops it at its next await,
    // i.e. between export/import pages.
    let end = tokio::select! {
        result = tokio::time::timeout(timeout, copy) => match result {
            Ok(result) => CopyEnd::Finished(result),
            Err(_) => CopyEnd::TimedOut,
        },
        () = cancel.notified() => CopyEnd::Cancelled,
    };
    match end {
        CopyEnd::Cancelled => {
            log::info!("{LOG_PREFIX} job={job_id} cancelled");
            update_job(&job_id, |j| j.state = "cancelled".to_string());
        }
        CopyEnd::TimedOut => fail_job(
            &job_id,
            format!(
                "the migration timed out after {}s; the active engine was not changed",
                timeout.as_secs()
            ),
        ),
        CopyEnd::Finished(Err(error)) => fail_job(&job_id, classify_engine_error(&error)),
        CopyEnd::Finished(Ok(report)) if report.failed > 0 => {
            let first = report.errors.first().map_or("", String::as_str);
            fail_job(
                &job_id,
                classify_engine_message(&format!(
                    "{} of {} records could not be copied; the active engine was not changed. {first}",
                    report.failed, report.records
                )),
            );
        }
        CopyEnd::Finished(Ok(report)) => {
            update_job(&job_id, |j| j.copied = report.records);
            match commit().await {
                Ok(()) => {
                    log::info!(
                        "{LOG_PREFIX} job={job_id} done records={} imported={} skipped={}",
                        report.records,
                        report.imported,
                        report.skipped
                    );
                    update_job(&job_id, |j| {
                        j.state = "done".to_string();
                        j.note = Some(WRITES_DURING_COPY_NOTE.to_string());
                    });
                }
                Err(error) => fail_job(&job_id, classify_engine_message(&error)),
            }
        }
    }
}

/// Shown on a finished job: what the copy cannot promise.
const WRITES_DURING_COPY_NOTE: &str =
    "Memories written while the copy was running may not have been \
     copied. The previous engine still holds them; migrate again to copy any that are missing.";

/// Spawn [`run_job`] under a supervisor so a panic fails the job instead of
/// leaving it "running" forever.
fn spawn_job<F, Fut>(
    job_id: String,
    source: Arc<dyn crate::memory::api::provider::MemoryProvider>,
    target: Arc<dyn crate::memory::api::provider::MemoryProvider>,
    timeout: Duration,
    commit: F,
) where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = Result<(), String>> + Send + 'static,
{
    let worker_id = job_id.clone();
    let worker = tokio::spawn(CoreContext::propagate(run_job(
        worker_id, source, target, timeout, commit,
    )));
    tokio::spawn(async move {
        if let Err(join_error) = worker.await {
            let reason = if join_error.is_panic() {
                "the migration task panicked; the active engine was not changed"
            } else {
                "the migration task was aborted; the active engine was not changed"
            };
            fail_job(&job_id, reason.to_string());
        }
    });
}

/// `memory.engine_migrate`.
pub async fn memory_engine_migrate(
    params: MigrateParams,
) -> Result<Outcome<MigrateStarted>, String> {
    let config: Config = crate::config::rpc::load_config_with_timeout().await?;
    let prepared = prepare_target(params.to)?;

    let source_binding = binding::for_config(&config)?;
    if source_binding.driver_id() == prepared.id && source_binding.fallback().is_none() {
        return Err(format!("memory already uses engine '{}'", prepared.id));
    }
    if source_binding.disables_memory() {
        return Err("memory is disabled (driver 'null'); there is nothing to migrate".to_string());
    }
    let source = Arc::clone(source_binding.provider());

    // Build the target before the job exists, so a bad endpoint, a missing key
    // or a missing backend transport answers this call instead of a poll.
    let target = if prepared.id == MODULE_ID {
        let mut cfg = config.subsystems.memory.clone();
        cfg.driver = MODULE_ID.to_string();
        Arc::clone(binding::for_subtree(&config.workspace_dir, "memory", &cfg)?.provider())
    } else {
        build_target_provider(&config, &prepared)?
    };

    // Registering the job under the switch lock keeps it from interleaving with
    // an `engine_set` that has already checked for running jobs.
    let job_id = {
        let _switch = SWITCH_LOCK.lock().await;
        start_job()?
    };
    log::info!(
        "{LOG_PREFIX} job={job_id} started source='{}' target='{}'",
        source_binding.driver_id(),
        prepared.id
    );

    spawn_job(
        job_id.clone(),
        source,
        target,
        migrate_timeout(),
        move || async move { commit_engine(&prepared).await.map(|_| ()) },
    );

    Ok(Outcome::new(MigrateStarted { job_id }, vec![]))
}

/// `memory.engine_migrate_cancel`.
pub async fn memory_engine_migrate_cancel(
    params: MigrateCancelParams,
) -> Result<Outcome<MigrateCancelled>, String> {
    let job_id = params.job_id.trim();
    let running = jobs()
        .get(job_id)
        .map(|j| j.state == "running")
        .ok_or_else(|| "unknown migration job".to_string())?;
    let cancelled = running && {
        if let Some(signal) = cancels().get(job_id) {
            signal.notify_one();
            true
        } else {
            false
        }
    };
    Ok(Outcome::new(MigrateCancelled { cancelled }, vec![]))
}

/// `memory.engine_migrate_status`.
pub async fn memory_engine_migrate_status(
    params: MigrateStatusParams,
) -> Result<Outcome<MigrateStatus>, String> {
    let status = jobs()
        .get(params.job_id.trim())
        .cloned()
        .ok_or_else(|| "unknown migration job".to_string())?;
    Ok(Outcome::new(status, vec![]))
}

#[cfg(test)]
#[path = "engine_migrate_tests.rs"]
mod tests;
