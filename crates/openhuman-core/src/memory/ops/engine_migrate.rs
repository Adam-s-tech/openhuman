//! `memory.engine_migrate` / `memory.engine_migrate_status` — copy everything
//! from the active engine into another, then switch to it.
//!
//! The copy is `tinymemory::migrate::copy_all`: the keyed records, then each
//! family both engines serve — document details, goals, the learned profile,
//! the conversation history — and, when the caller asks, the ingested content,
//! re-sent raw so the new engine rebuilds its summary tree. It is engine-neutral,
//! at-least-once and non-destructive (the source keeps its data, and the target
//! skips what it already holds), so a failed or repeated migration is safe and
//! the previous engine stays usable. The switch is committed only after the copy
//! finished with nothing failed; on any failure the active engine is untouched.
//! Re-sent content is the exception: it stays in the previous engine and the
//! sync that produced it can bring it again, so a piece the new engine refuses
//! is reported in the job's note rather than holding the switch back.
//!
//! The host decides what the replay leaves out. A target the host syncs sources
//! into itself gets every reader-based source again from scratch (see
//! `memory::sources::hosted_sync`), so their chunks are not re-sent.
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
    /// Whether to re-send ingested content so the new engine rebuilds its
    /// summary tree from it. Hosted memory bills for the content it reads
    /// again, which is why it is a choice. Defaults to `true`.
    #[serde(default = "replay_by_default")]
    pub replay_content: bool,
}

fn replay_by_default() -> bool {
    true
}

/// The prefix of a reader-based source's chunk ids, `mem_src:<source>:<item>`
/// (see `tinymemory_api::sync_events::extract_mem_src_id`).
const READER_SOURCE_PREFIX: &str = "mem_src:";

/// What a copy carries beyond the keyed records, as the host decides it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct CopyChoices {
    replay_content: bool,
    skip_source_prefixes: Vec<String>,
}

/// The choices for a copy into a target: one the host syncs sources into
/// itself (`host_synced`) gets every reader-based source again from scratch,
/// so the replay leaves those out rather than send them twice.
fn copy_choices(replay_content: bool, host_synced: bool) -> CopyChoices {
    let skip_source_prefixes = if host_synced {
        vec![READER_SOURCE_PREFIX.to_string()]
    } else {
        Vec::new()
    };
    CopyChoices {
        replay_content,
        skip_source_prefixes,
    }
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
    /// The step under way — `records`, `documents`, `goals`, `profile`,
    /// `episodic` or `content` — or the last one once the copy ended. `None`
    /// before the first step reports.
    pub step: Option<String>,
    /// Items the step under way has read so far.
    pub step_read: usize,
    /// Items the step under way has written so far.
    pub step_written: usize,
    /// What each step after the records did, once the copy finished.
    pub steps: Vec<MigrateStepStatus>,
}

/// What one step after the keyed records did.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MigrateStepStatus {
    /// `documents`, `goals`, `profile`, `episodic` or `content`.
    pub step: String,
    /// Why the step did not run — an engine does not serve what it needs, or
    /// the caller chose not to re-send content. `None` when it ran.
    pub skipped_because: Option<String>,
    pub read: usize,
    pub written: usize,
    /// Items the new engine already held as the old one has them.
    pub unchanged: usize,
    pub failed: usize,
    /// Why items failed, naming them and never their content; at most 20.
    pub errors: Vec<String>,
}

/// Cancel signals of the jobs in [`JOBS`].
struct CancelSlot {
    notify: Arc<Notify>,
    requested: bool,
}

static CANCELS: OnceLock<Mutex<HashMap<String, CancelSlot>>> = OnceLock::new();

fn cancels() -> std::sync::MutexGuard<'static, HashMap<String, CancelSlot>> {
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
            step: None,
            step_read: 0,
            step_written: 0,
            steps: Vec::new(),
        },
    );
    cancels().insert(
        job_id.clone(),
        CancelSlot {
            notify: Arc::new(Notify::new()),
            requested: false,
        },
    );
    Ok(job_id)
}

/// What a finished copy reports.
struct CopyReport {
    records: usize,
    imported: usize,
    skipped: usize,
    failed: usize,
    errors: Vec<String>,
    steps: Vec<MigrateStepStatus>,
}

/// One progress report from the copy: the step under way and its counts.
#[derive(Clone, Copy, Debug)]
struct CopyTick {
    step: &'static str,
    read: usize,
    written: usize,
}

#[cfg(feature = "memory-remote")]
async fn run_copy(
    source: &dyn crate::memory::api::provider::MemoryProvider,
    target: &dyn crate::memory::api::provider::MemoryProvider,
    choices: &CopyChoices,
    on_progress: impl FnMut(CopyTick),
) -> anyhow::Result<CopyReport> {
    let mut on_progress = on_progress;
    let options = tinymemory::migrate::CopyOptions {
        replay_content: choices.replay_content,
        skip_source_prefixes: choices.skip_source_prefixes.clone(),
        ..tinymemory::migrate::CopyOptions::default()
    };
    let report = tinymemory::migrate::copy_all(source, target, &options, |p| {
        on_progress(CopyTick {
            step: p.step.as_str(),
            read: p.read,
            written: p.written,
        });
    })
    .await?;
    Ok(CopyReport {
        records: report.records.records,
        imported: report.records.imported,
        skipped: report.records.skipped,
        failed: report.records.failed,
        errors: report.records.errors,
        steps: report
            .steps
            .into_iter()
            .map(|step| MigrateStepStatus {
                step: step.step.as_str().to_string(),
                skipped_because: step.skipped_because,
                read: step.read,
                written: step.written,
                unchanged: step.unchanged,
                failed: step.failed,
                errors: step.errors,
            })
            .collect(),
    })
}

#[cfg(not(feature = "memory-remote"))]
async fn run_copy(
    _source: &dyn crate::memory::api::provider::MemoryProvider,
    _target: &dyn crate::memory::api::provider::MemoryProvider,
    _choices: &CopyChoices,
    _on_progress: impl FnMut(CopyTick),
) -> anyhow::Result<CopyReport> {
    anyhow::bail!("memory engine migration is not compiled into this build")
}

/// The step whose failures hold the switch back, if any: every step but the
/// content replay, which only re-sends what the previous engine keeps.
fn blocking_failure(steps: &[MigrateStepStatus]) -> Option<&MigrateStepStatus> {
    steps
        .iter()
        .find(|step| step.step != "content" && step.failed > 0)
}

/// The caveats on a finished copy: writes made while it ran, and re-sent
/// content the new engine refused.
fn finished_note(steps: &[MigrateStepStatus]) -> String {
    let mut note = WRITES_DURING_COPY_NOTE.to_string();
    if let Some(content) = steps
        .iter()
        .find(|step| step.step == "content" && step.failed > 0)
    {
        note.push_str(&format!(
            " {} pieces of synced content could not be re-sent; the previous engine still holds them, and their next sync can bring them again.",
            content.failed
        ));
    }
    note
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
    choices: CopyChoices,
    timeout: Duration,
    commit: F,
) where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<(), String>>,
{
    let cancel = cancels()
        .get(&job_id)
        .map(|slot| Arc::clone(&slot.notify))
        .unwrap_or_default();
    let progress_id = job_id.clone();
    let copy = run_copy(source.as_ref(), target.as_ref(), &choices, move |tick| {
        update_job(&progress_id, |j| {
            if j.step.as_deref() != Some(tick.step) {
                log::info!("{LOG_PREFIX} job={progress_id} step={} started", tick.step);
            }
            if tick.step == "records" {
                j.copied = tick.read;
            }
            j.step = Some(tick.step.to_string());
            j.step_read = tick.read;
            j.step_written = tick.written;
        });
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
            let steps = report.steps.clone();
            update_job(&job_id, |j| j.steps = steps);
            for step in &report.steps {
                log::info!(
                    "{LOG_PREFIX} job={job_id} step={} read={} written={} unchanged={} failed={} skipped_because={:?}",
                    step.step,
                    step.read,
                    step.written,
                    step.unchanged,
                    step.failed,
                    step.skipped_because
                );
            }
            if let Some(step) = blocking_failure(&report.steps) {
                let first = step.errors.first().map_or("", String::as_str);
                fail_job(
                    &job_id,
                    classify_engine_message(&format!(
                        "{} of {} {} items could not be copied; the active engine was not changed. {first}",
                        step.failed, step.read, step.step
                    )),
                );
                return;
            }
            // The copy is past its cancellation point. Removing the signal
            // makes concurrent cancel requests report `cancelled: false`.
            if cancels().remove(&job_id).is_some_and(|slot| slot.requested) {
                update_job(&job_id, |j| j.state = "cancelled".to_string());
                return;
            }
            update_job(&job_id, |j| j.copied = report.records);
            match commit().await {
                Ok(()) => {
                    log::info!(
                        "{LOG_PREFIX} job={job_id} done records={} imported={} skipped={}",
                        report.records,
                        report.imported,
                        report.skipped
                    );
                    let note = finished_note(&report.steps);
                    update_job(&job_id, |j| {
                        j.state = "done".to_string();
                        j.note = Some(note);
                    });
                }
                Err(error) => fail_job(&job_id, classify_engine_message(&error)),
            }
        }
    }
}

/// Shown on a finished job: what the copy cannot promise.
const WRITES_DURING_COPY_NOTE: &str =
    "Memories written while the copy was running may not have been copied. The previous engine still holds them; this migration does not provide a second-pass delta copy.";

/// Spawn [`run_job`] under a supervisor so a panic fails the job instead of
/// leaving it "running" forever.
fn spawn_job<F, Fut>(
    job_id: String,
    source: Arc<dyn crate::memory::api::provider::MemoryProvider>,
    target: Arc<dyn crate::memory::api::provider::MemoryProvider>,
    choices: CopyChoices,
    timeout: Duration,
    commit: F,
) where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = Result<(), String>> + Send + 'static,
{
    let worker_id = job_id.clone();
    let worker = tokio::spawn(CoreContext::propagate(run_job(
        worker_id, source, target, choices, timeout, commit,
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
    let choices = copy_choices(
        params.replay_content,
        crate::memory::sources::hosted_sync::host_synced(target.as_ref()),
    );
    log::info!(
        "{LOG_PREFIX} job={job_id} started source='{}' target='{}' replay_content={} skip_sources={:?}",
        source_binding.driver_id(),
        prepared.id,
        choices.replay_content,
        choices.skip_source_prefixes
    );

    spawn_job(
        job_id.clone(),
        source,
        target,
        choices,
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
        if let Some(slot) = cancels().get_mut(job_id) {
            slot.requested = true;
            slot.notify.notify_one();
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
