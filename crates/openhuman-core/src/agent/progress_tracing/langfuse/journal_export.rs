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
