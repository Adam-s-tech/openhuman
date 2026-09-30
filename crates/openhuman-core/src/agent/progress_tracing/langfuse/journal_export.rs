//! Journal export: pushes a run's durable tinyagents observations through the
//! tinyagents Langfuse exporter. Lineage stamping, trace attribution, content
//! gating and the run-total fold are upstream
//! (`tinyagents_harness::observability::trace_export::journal_export`); this
//! module resolves the backend URL and credential and sends the chunks.

use tinyagents_harness::observability::trace_export::ingestion_batch::{
    split_ingestion_batch, LANGFUSE_MAX_BATCH_EVENTS,
};
use tinyagents_harness::observability::trace_export::journal_export::{
    insert_run_telemetry_generation, observations_for_export, trace_config_from_context,
    trace_ctx_with_run_lineage, RunTotals,
};
use tinyagents_harness::observability::trace_export::TraceContext;
use tinyagents_harness::observability::{AgentObservation, LangfuseClient};
use tinyagents_session::run_ledger::RunTelemetry;

use crate::config::Config;
use crate::security::credentials::session_support::direct_backend_credential;

use super::{environment_for_base, ingestion_url, skip_push, LOG_TARGET, PUSH_TIMEOUT};

/// Project the run-ledger aggregate onto the upstream run-total shape.
fn run_totals(telemetry: &RunTelemetry) -> RunTotals {
    RunTotals {
        run_id: telemetry.run_id.clone(),
        input_tokens: telemetry.input_tokens,
        output_tokens: telemetry.output_tokens,
        cached_input_tokens: telemetry.cached_input_tokens,
        cost_usd: telemetry.cost_usd,
        tool_count: telemetry.tool_count,
        model: telemetry.model.clone(),
        provider: telemetry.provider.clone(),
        error: telemetry.error.clone(),
    }
}

/// Push durable journal observations through the tinyagents crate Langfuse
/// exporter. The journal is already redacted before persistence, and this
/// exporter additionally strips model/tool payloads unless `capture_content`
/// is explicitly enabled.
/// Whether [`push_observations`] would actually send for `config`: the same
/// gates it checks, for a caller that must read a journal and build
/// observations first. Without a live session (unit tests, a signed-out or
/// embedder host) or on a skipped environment, that work is discarded
/// anyway — and reading a whole child journal is not free.
pub(crate) fn journal_push_ready(config: &Config) -> bool {
    let url = ingestion_url(config);
    !skip_push(environment_for_base(&url))
        && url.starts_with("http")
        && matches!(
            direct_backend_credential(config, "langfuse journal push"),
            Some(crate::security::credentials::session_support::BackendCredential::Session(_))
        )
}

pub(crate) async fn push_observations(
    config: &Config,
    trace_ctx: &TraceContext,
    observations: &[AgentObservation],
    run_telemetry: Option<&RunTelemetry>,
) -> Result<(), String> {
    if observations.is_empty() {
        return Ok(());
    }
    let url = ingestion_url(config);
    // Same gate, same reasons as `push_spans` — both entry points are on the
    // per-turn path, so both skip before doing any work.
    let environment = environment_for_base(&url);
    if skip_push(environment) {
        return Ok(());
    }
    if !url.starts_with("http") {
        return Err(format!(
            "could not resolve Langfuse ingestion URL from backend host (got {url:?})"
        ));
    }
    // No TinyHumans connection, or no usable credential (signed out, offline
    // local session): a configured state, so skip quietly rather than failing
    // every turn's push.
    let token = match direct_backend_credential(config, "langfuse journal push") {
        Some(crate::security::credentials::session_support::BackendCredential::Session(token)) => {
            token
        }
        _ => return Ok(()),
    };
    // Stamp the run lineage from the run's own observations so a spawned
    // sub-agent's trace links back to its parent turn (#4657).
    let brand = crate::agent::progress_tracing::export_brand();
    let trace_ctx = trace_ctx_with_run_lineage(trace_ctx, observations);
    let trace = trace_config_from_context(&trace_ctx, environment, &brand);
    let observation_count = observations.len();
    let observations = observations_for_export(&trace_ctx, observations);

    tracing::debug!(
        target: LOG_TARGET,
        "[agent-tracing] pushing {observation_count} journal observations to Langfuse at {url}"
    );

    let client = LangfuseClient::proxy(url, token)
        .map_err(|err| format!("Langfuse client setup failed: {err}"))?;
    let mut payload = client
        .build_ingestion_batch(trace, observations.as_ref())
        .map_err(|err| format!("Langfuse journal batch build failed: {err}"))?;
    if insert_run_telemetry_generation(
        &mut payload,
        run_telemetry.map(run_totals).as_ref(),
        &brand,
    ) {
        tracing::debug!(
            target: LOG_TARGET,
            "[agent-tracing] added run telemetry aggregate to Langfuse journal batch"
        );
    } else {
        tracing::debug!(
            target: LOG_TARGET,
            "[agent-tracing] no run telemetry aggregate added to Langfuse journal batch"
        );
    }
    // Langfuse caps a single ingestion request at 500 events; a large run (e.g.
    // one that spawns sub-agents) can far exceed that and previously had its
    // ENTIRE trace rejected with a 400. Send in <=500-event chunks instead.
    for chunk in split_ingestion_batch(payload, LANGFUSE_MAX_BATCH_EVENTS) {
        tokio::time::timeout(PUSH_TIMEOUT, client.send_batch(chunk))
            .await
            .map_err(|_| format!("Langfuse journal push timed out after {PUSH_TIMEOUT:?}"))?
            .map_err(|err| format!("Langfuse journal ingestion failed: {err}"))?;
    }

    tracing::debug!(
        target: LOG_TARGET,
        "[agent-tracing] pushed {observation_count} journal observations to Langfuse"
    );
    Ok(())
}
