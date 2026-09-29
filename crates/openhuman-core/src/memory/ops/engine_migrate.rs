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

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use serde::{Deserialize, Serialize};

use super::engine::{
    build_target_provider, classify_engine_error, classify_engine_message, commit_engine,
    prepare_target, EngineTargetParams,
};
use crate::config::schema::Config;
use crate::core::runtime::context::CoreContext;
use crate::memory::binding::{self, MODULE_ID};
use crate::rpc::RpcOutcome;

const LOG_PREFIX: &str = "[memory:engine-migrate]";
/// Finished jobs kept for status polling.
const MAX_FINISHED_JOBS: usize = 16;

/// Parameters of `memory.engine_migrate`.
#[derive(Clone, Deserialize)]
pub struct MigrateParams {
    pub to: EngineTargetParams,
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
    /// `running` | `done` | `failed`.
    pub state: String,
    /// Records read from the source and handed to the target so far.
    pub copied: usize,
    /// Total records when known; the export cursor does not report one.
    pub total: Option<usize>,
    pub error: Option<String>,
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
    }
    let job_id = uuid::Uuid::new_v4().to_string();
    map.insert(
        job_id.clone(),
        MigrateStatus {
            state: "running".to_string(),
            copied: 0,
            total: None,
            error: None,
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

/// `memory.engine_migrate`.
pub async fn memory_engine_migrate(
    params: MigrateParams,
) -> Result<RpcOutcome<MigrateStarted>, String> {
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

    let job_id = start_job()?;
    log::info!(
        "{LOG_PREFIX} job={job_id} started source='{}' target='{}'",
        source_binding.driver_id(),
        prepared.id
    );

    let task_job_id = job_id.clone();
    tokio::spawn(CoreContext::propagate(async move {
        let progress_id = task_job_id.clone();
        let copied = run_copy(source.as_ref(), target.as_ref(), move |records| {
            update_job(&progress_id, |j| j.copied = records);
        })
        .await;
        match copied {
            Err(error) => fail_job(&task_job_id, classify_engine_error(&error)),
            Ok(report) if report.failed > 0 => {
                let first = report.errors.first().map_or("", String::as_str);
                fail_job(
                    &task_job_id,
                    classify_engine_message(&format!(
                        "{} of {} records could not be copied; the active engine was not changed. {first}",
                        report.failed, report.records
                    )),
                );
            }
            Ok(report) => {
                update_job(&task_job_id, |j| j.copied = report.records);
                match commit_engine(config, &prepared).await {
                    Ok(_) => {
                        log::info!(
                            "{LOG_PREFIX} job={task_job_id} done records={} imported={} skipped={}",
                            report.records,
                            report.imported,
                            report.skipped
                        );
                        update_job(&task_job_id, |j| j.state = "done".to_string());
                    }
                    Err(error) => fail_job(&task_job_id, classify_engine_message(&error)),
                }
            }
        }
    }));

    Ok(RpcOutcome::new(MigrateStarted { job_id }, vec![]))
}

/// `memory.engine_migrate_status`.
pub async fn memory_engine_migrate_status(
    params: MigrateStatusParams,
) -> Result<RpcOutcome<MigrateStatus>, String> {
    let status = jobs()
        .get(params.job_id.trim())
        .cloned()
        .ok_or_else(|| "unknown migration job".to_string())?;
    Ok(RpcOutcome::new(status, vec![]))
}

#[cfg(test)]
#[path = "engine_migrate_tests.rs"]
mod tests;
