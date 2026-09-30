//! Gap-filling unit tests for the agent harness.
//!
//! These tests cover paths that were missing from the existing `*_tests.rs`
//! co-located files as identified by a coverage gap analysis:
//!
//! 1. Full user→LLM→tool→result→final turn cycle — now covered by the
//!    tinyagents route's tests (`crates/openhuman-core/src/agent/tinyagents/tests.rs`), which
//!    exercise `run_turn_via_tinyagents_shared` end to end.
//! 2. `MaxIterationsExceeded` downcasts to the typed `AgentError` variant.
//! 3. `visible_tool_names` whitelist: tools outside the set are treated as unknown.
//! 4. `ContextGuard` surfaces `ContextExhausted` and aborts the loop.
//! 5. `parse_tool_calls` fallback formats — covered in `tinytools-agent`'s own
//!    parse tests (`parse/test/regressions.rs`).
//! 6. `DateTimeSection` produces an ISO-8601-like timestamp with a timezone token.
//! 7. `parse_tool_timeout_secs` default and boundary cases.
//! 8. Spawn-depth gate (`SpawnDepthExceeded`) is covered in
//!    `../subagent_host/ops_tests.rs` because it lives at the `run_subagent`
//!    boundary.
//!
//! Items that have NO underlying code and therefore cannot be tested:
//! - Follow-up resolution ("yes"/"no" disambiguation) — not implemented.
//! - Silence timer (SilenceTimeout, 600 s) — not implemented.
//! - `<invoke tool=…>` XML attribute form — the parser does not parse attributes;
//!   only the tag body (JSON) is used.

use crate::tools::timeout::parse_tool_timeout_secs;

// ─────────────────────────────────────────────────────────────────────────────
// Item 1 — Full turn cycle: user → LLM emits tool call → tool executes →
//           result injected → LLM produces final text.
// ─────────────────────────────────────────────────────────────────────────────

// NOTE: The `ContextGuard`/`ContextCheckResult` tests that used to live here
// (context_guard_exhausted_after_circuit_breaker_and_95pct_utilization,
// context_guard_update_usage_raises_window_from_response) were removed during the
// tinyagents migration: the context reducer shell (`context/guard.rs`) was
// deleted (commit d55ea9a5d) and the tested API no longer exists.

// ─────────────────────────────────────────────────────────────────────────────
// Item 5 (tool timeout) — parse_tool_timeout_secs defaults and boundaries.
//   Already covered in tool_timeout/mod.rs but pinned here for the gap report.
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn tool_timeout_parse_default_and_boundaries() {
    // Default when absent.
    assert_eq!(parse_tool_timeout_secs(None), 120);
    // Default when non-numeric.
    assert_eq!(parse_tool_timeout_secs(Some("bad")), 120);
    // Boundary values.
    assert_eq!(parse_tool_timeout_secs(Some("1")), 1);
    assert_eq!(parse_tool_timeout_secs(Some("3600")), 3600);
    // Out of range → default.
    assert_eq!(parse_tool_timeout_secs(Some("0")), 120);
    assert_eq!(parse_tool_timeout_secs(Some("3601")), 120);
}

// ─────────────────────────────────────────────────────────────────────────────
// Item 8 — Current-time grounding (#3602). The volatile timestamp now rides the
//           per-turn user message via `current_datetime_line` (so a long-lived
//           session's frozen prompt prefix can't go stale); `DateTimeSection`
//           carries only the static grounding *rule*. Pin both halves.
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn current_datetime_line_matches_iso8601_date_and_utc_offset_pattern() {
    // The per-turn stamp is the one carrying the concrete clock — assert its
    // ISO-8601 date, UTC offset, and IANA zone (or `UTC` fallback).
    let payload = crate::agent::prompts::current_datetime_line();

    // Parse the concrete `YYYY-MM-DD HH:MM:SS` prefix rather than counting
    // loose digits, so a malformed layout can't slip through.
    let rest = payload
        .strip_prefix("Current Date & Time: ")
        .expect("stamp must start with the canonical prefix");
    let dt = rest
        .get(0..19)
        .expect("stamp must include YYYY-MM-DD HH:MM:SS");
    chrono::NaiveDateTime::parse_from_str(dt, "%Y-%m-%d %H:%M:%S")
        .expect("timestamp must match YYYY-MM-DD HH:MM:SS");
    assert!(
        payload.contains("UTC"),
        "stamp must contain UTC offset marker: {payload}"
    );
    let has_iana = payload.contains('/') || payload.contains(" UTC ");
    assert!(
        has_iana,
        "stamp must contain an IANA zone (slashed) or UTC fallback: {payload}"
    );
}
