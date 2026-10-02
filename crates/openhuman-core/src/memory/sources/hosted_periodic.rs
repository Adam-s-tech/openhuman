//! Scheduled syncs of the local memory sources when the host syncs them.
//!
//! With the embedded engine, the TinyMemory module refreshes folder, GitHub,
//! RSS and web-page sources on a schedule of its own. A driver that keeps what
//! it is sent and runs no source pipeline (hosted memory; see
//! [`super::hosted_sync`]) has no such schedule, so without this loop those
//! sources would change only when the user pressed Sync. It runs each due
//! source through the same host sync the Sync button reaches, recorded the
//! same way, and it keeps the engine loop's rules:
//!
//! - the cadence is the user's `memory_sync_interval_secs`, never shorter than
//!   a day, and `0` means manual only;
//! - a source is due when its last run from this loop, or its last successful
//!   run in the host's run log, is older than that — so a restart does not
//!   sync everything again;
//! - nothing runs while the user is signed out or has turned background work
//!   off;
//! - nothing runs unless the driver bound at that tick is one the host syncs,
//!   because the engine can be switched while the app runs.
//!
//! Connector sources are not scheduled here; the periodic Composio loop owns
//! them. Nor are conversations and searches, which the engine's loop does not
//! schedule either.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock, PoisonError};
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};

use crate::config::DEFAULT_MEMORY_SYNC_INTERVAL_SECS;
use crate::cron::scheduler_gate::PauseReason;
use crate::memory::api::provider::sync::SyncAuditEntry;
use crate::memory::sources::hosted_sync;
use crate::memory::sources::rpc::driver_run;
use crate::memory::sources::run_history;
use crate::memory::sources::types::{MemorySourceEntry, SourceKind};

/// How often the loop looks for due sources. The per-source cadence, a day at
/// least, bounds how often each one runs; this bounds how late past due it
/// can start.
const TICK: Duration = Duration::from_secs(1200);

/// Guards [`start_hosted_periodic_sync`] against a second loop.
static STARTED: OnceLock<()> = OnceLock::new();

/// When this loop last started each source, by source id.
fn last_fired() -> &'static Mutex<HashMap<String, Instant>> {
    static FIRED: OnceLock<Mutex<HashMap<String, Instant>>> = OnceLock::new();
    FIRED.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Start the loop. Idempotent. Each tick does nothing unless the driver bound
/// at that moment is one the host syncs, so an embedded engine, whose module
/// schedules these sources itself, is left to it.
pub fn start_hosted_periodic_sync() {
    if STARTED.set(()).is_err() {
        tracing::debug!("[memory_sources:hosted_periodic] loop already running");
        return;
    }
    tracing::info!(
        tick_secs = TICK.as_secs(),
        "[memory_sources:hosted_periodic] starting the scheduled sync of local sources"
    );
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(TICK);
        // The first tick fires at once; skip it so startup is not spent
        // syncing before the user has even signed in.
        ticker.tick().await;
        loop {
            ticker.tick().await;
            match run_one_tick().await {
                Ok(0) => {}
                Ok(ran) => tracing::debug!(ran, "[memory_sources:hosted_periodic] tick done"),
                Err(error) => {
                    tracing::warn!(%error, "[memory_sources:hosted_periodic] tick failed");
                }
            }
        }
    });
}

/// One pass: run every due source, one after another, and answer how many
/// ran.
async fn run_one_tick() -> Result<usize, String> {
    if let Some(reason) = pause_reason() {
        tracing::debug!(
            ?reason,
            "[memory_sources:hosted_periodic] background work is paused; skipping"
        );
        return Ok(0);
    }
    let config = crate::config::rpc::load_config_with_timeout().await?;
    let binding = crate::memory::binding::for_config(&config)?;
    let provider = binding.provider();
    if !hosted_sync::host_synced(provider.as_ref()) {
        return Ok(0);
    }
    let Some(sink) = provider.as_sources() else {
        return Ok(0);
    };
    let Some(interval_secs) = effective_interval_secs(config.memory_sync_interval_secs) else {
        tracing::debug!("[memory_sources:hosted_periodic] manual-only; skipping");
        return Ok(0);
    };
    let history = match run_history::read_runs(&config.workspace_dir, run_history::KEEP_ROWS) {
        Ok(entries) => Some(last_success_by_source(&entries)),
        Err(error) => {
            tracing::warn!(
                %error,
                "[memory_sources:hosted_periodic] run log unreadable; only sources this \
                 process has run are considered"
            );
            None
        }
    };
    let sources = crate::memory::sources::registry::list_sources_in(&config)?;
    let fired: HashMap<String, Duration> = last_fired()
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .iter()
        .map(|(source_id, at)| (source_id.clone(), at.elapsed()))
        .collect();
    let due = due_sources(
        &sources,
        interval_secs,
        &fired,
        history.as_ref(),
        Utc::now(),
    );
    if due.is_empty() {
        return Ok(0);
    }

    let mut ran = 0;
    for source in due {
        // Recorded before the run, success or not, so a source that keeps
        // failing waits out its interval rather than retrying every tick.
        last_fired()
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(source.id.clone(), Instant::now());
        tracing::info!(
            source_id = %source.id,
            kind = source.kind.as_str(),
            interval_secs,
            "[memory_sources:hosted_periodic] syncing a due source"
        );
        let result = driver_run::run_recorded(
            &config,
            &source.id,
            Some(source),
            || hosted_sync::run(&config, source, sink),
            |error| error.to_string(),
            driver_run::bus_stage_publisher(driver_run::PERIODIC, &source.id, source.kind.as_str()),
        )
        .await;
        if let Err(error) = result {
            tracing::warn!(
                source_id = %source.id,
                %error,
                "[memory_sources:hosted_periodic] sync failed; retried when next due"
            );
        }
        ran += 1;
    }
    Ok(ran)
}

/// Why background work is paused, when it is for a reason this loop honours:
/// the user turned it off, or is signed out. Battery and CPU pressure throttle
/// LLM work; a sync that is due a day at a time is not held back by them.
fn pause_reason() -> Option<PauseReason> {
    let reason = crate::cron::scheduler_gate::current_policy().pause_reason()?;
    matches!(reason, PauseReason::UserDisabled | PauseReason::SignedOut).then_some(reason)
}

/// The cadence in seconds for the user's setting, or `None` for manual only.
/// Never shorter than the default day, as the engine's loop rules.
fn effective_interval_secs(configured: Option<u64>) -> Option<u64> {
    match configured {
        Some(0) => None,
        Some(secs) => Some(secs.max(DEFAULT_MEMORY_SYNC_INTERVAL_SECS)),
        None => Some(DEFAULT_MEMORY_SYNC_INTERVAL_SECS),
    }
}

/// Source kinds this loop schedules.
fn is_scheduled_kind(kind: &SourceKind) -> bool {
    hosted_sync::serves(kind)
}

/// The newest successful run of each scheduled source in the run log.
fn last_success_by_source(entries: &[SyncAuditEntry]) -> HashMap<String, DateTime<Utc>> {
    let mut newest: HashMap<String, DateTime<Utc>> = HashMap::new();
    for entry in entries.iter().filter(|entry| {
        entry.success
            && matches!(
                entry.source_kind.as_str(),
                "folder" | "github_repo" | "rss_feed" | "web_page"
            )
    }) {
        newest
            .entry(entry.source_id.clone())
            .and_modify(|at| *at = (*at).max(entry.timestamp))
            .or_insert(entry.timestamp);
    }
    newest
}

/// The enabled, scheduled sources whose last run is at least `interval_secs`
/// old at `now`.
///
/// A source this process ran is timed from that run (`fired` holds how long
/// ago each started). Otherwise it is timed from its last success in the run
/// log, and one with none there has never synced and is due. Without a
/// readable log, only sources this process ran can be timed, so the rest wait
/// rather than all running at once.
fn due_sources<'a>(
    sources: &'a [MemorySourceEntry],
    interval_secs: u64,
    fired: &HashMap<String, Duration>,
    history: Option<&HashMap<String, DateTime<Utc>>>,
    now: DateTime<Utc>,
) -> Vec<&'a MemorySourceEntry> {
    let interval = Duration::from_secs(interval_secs);
    sources
        .iter()
        .filter(|source| source.enabled && is_scheduled_kind(&source.kind))
        .filter(|source| {
            let since = match (fired.get(&source.id), history) {
                (Some(elapsed), _) => Some(*elapsed),
                (None, Some(history)) => history
                    .get(&source.id)
                    .map(|at| (now - *at).to_std().unwrap_or_default()),
                (None, None) => return false,
            };
            since.is_none_or(|since| since >= interval)
        })
        .collect()
}

#[cfg(test)]
#[path = "hosted_periodic_tests.rs"]
mod tests;
