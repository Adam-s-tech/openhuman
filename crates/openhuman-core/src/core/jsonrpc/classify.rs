//! How a failed RPC is classified at the transport boundary.
//!
//! Two decisions hang off an error string: whether it is a confirmed OpenHuman
//! session expiry (which signs the user out, see
//! [`invoke_method`](super::invoke_method)), and how loudly the `/rpc` handler
//! reports it ([`classify_failure`]). Both are pure so they can be tested
//! without a server.

/// Helper to determine if an error message indicates an expired or invalid
/// OpenHuman backend session.
///
/// **Narrower than the previous implementation** (fixed in issue #2286):
///
/// The old predicate matched ANY `"401 + unauthorized"` pattern, which caused
/// downstream provider 401s (Discord bot token failures, BYO-key OpenAI /
/// Anthropic failures, Composio direct-mode errors) to clear the user's session
/// and log them out. The fix distinguishes between:
///
/// - **OpenHuman backend 401s** (`authed_json` in `crates/openhuman-core/src/api/rest.rs`): formatted
///   as `"{METHOD} /path failed (401 Unauthorized): {body}"`, e.g.
///   `"GET /teams failed (401 Unauthorized): {"success":false}"`. These always
///   start with an HTTP method verb followed by a space and a forward slash.
/// - **Provider / downstream 401s** (`api_error` in
///   `crates/openhuman-core/src/inference/provider/ops.rs`): formatted as
///   `"{ProviderName} API error (401 Unauthorized): {body}"` or
///   `"Discord API error: ... (401): Unauthorized"`. These start with a
///   provider name, NOT an HTTP method verb.
///
/// **What still triggers session expiry:**
/// - `"Session expired"` — explicit body text from the OpenHuman backend.
/// - `"no backend session token"` — pre-flight guard; auth profile is missing.
/// - `"session jwt required"` — local guard; JWT already cleared by a prior 401.
/// - `"SESSION_EXPIRED"` — scheduler-gate sentinel (exact case).
/// - HTTP-method-prefixed 401s (`GET /`, `POST /`, etc.) — backend path format.
///
/// **What no longer triggers session expiry (fixed in #2286):**
/// - Provider-prefixed 401s (`"Discord API error: ..."`, `"OpenAI API error ..."`)
/// - `"invalid token"` — too broad; also matches Discord / OAuth provider tokens.
///
/// Note: for inference-path OpenHuman backend 401s, `api_error` (in
/// `inference/provider/ops.rs` lines 479–497) ALREADY publishes `SessionExpired`
/// directly, so there is no regression if this predicate misses them — the
/// subscriber is idempotent and a harmless double-publish would still be correct.
pub(super) fn is_session_expired_error(msg: &str) -> bool {
    // Explicit session-expired markers from the OpenHuman backend / local
    // guards — delegated to the shared observability classifier so both the
    // Sentry expected-error pipeline and the JSON-RPC publish boundary stay
    // in lock-step.
    if crate::core::observability::is_session_expired_message(msg) {
        return true;
    }
    // OpenHuman backend path 401s via `authed_json`:
    // format is "{METHOD} /path failed (401 Unauthorized): {body}"
    // The HTTP-method prefix distinguishes these from provider-prefixed errors.
    // HEAD and OPTIONS are intentionally excluded — `authed_json` only issues
    // the five listed verbs (GET/POST/PUT/DELETE/PATCH) for REST JSON endpoints.
    let lower = msg.to_ascii_lowercase();
    if (lower.contains("401") && lower.contains("unauthorized"))
        && (msg.starts_with("GET /")
            || msg.starts_with("POST /")
            || msg.starts_with("PUT /")
            || msg.starts_with("DELETE /")
            || msg.starts_with("PATCH /"))
    {
        return true;
    }
    false
}

/// Detect auth-looking failures that are not specific enough to clear the
/// OpenHuman session. This is only for diagnostics; it must not feed the
/// `SessionExpired` publish path.
///
/// Matches a generic `401 Unauthorized` OR a bare `"invalid token"` string,
/// either of which can come from BYO-key providers, Composio, channels, or
/// other scoped downstream calls. Used exclusively for diagnostic logging
/// at the `invoke_method` call site so provider auth failures are visible
/// in the logs without being misclassified as session expiry.
pub(super) fn is_unconfirmed_unauthorized_error(msg: &str) -> bool {
    let lower = msg.to_ascii_lowercase();
    (lower.contains("401") && lower.contains("unauthorized")) || lower.contains("invalid token")
}

/// Returns `true` when the error is the wallet's "not configured yet" message.
///
/// Wallet-backed RPCs return
/// [`crate::web3::wallet::WALLET_NOT_CONFIGURED_MESSAGE`] before
/// setup. That is expected user state, not an internal failure.
///
/// Matched against the shared wallet constant (exact equality) so a wording
/// change in the wallet layer fails the coupling test in `classify_tests.rs`
/// rather than silently letting the noise back into Sentry.
#[cfg(feature = "http-server")]
pub(super) fn is_wallet_not_configured_error(msg: &str) -> bool {
    msg == crate::web3::wallet::WALLET_NOT_CONFIGURED_MESSAGE
}

/// How the `/rpc` handler reports a failed call, in priority order.
///
/// Only [`FailureDisposition::Unexpected`] is an error-level Sentry event. The
/// rest are either expected boundary conditions or already reported elsewhere.
/// The JSON-RPC error returned to the caller is the same for every variant,
/// except that [`FailureDisposition::UsageProbeBackoff`] replaces the message.
#[cfg(feature = "http-server")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum FailureDisposition {
    /// The controller's structured envelope set `expected_user_state` (stale
    /// thread refs and similar). Domains that surface their own expected
    /// user-state errors skip Sentry here uniformly.
    ExpectedUserState,
    /// A wallet-backed RPC cannot run before wallet setup. This is expected
    /// user state, not an internal failure.
    WalletNotConfigured,
    /// Param-validation failures ("unknown param 'x' for ns.fn", "missing
    /// required param 'x'", "invalid params: …") are pure boundary
    /// mismatches: either the caller is a frontend on a different release than
    /// the running core (OPENHUMAN-TAURI-20: v0.53.22 UI shipped `api_key`
    /// before the matching schema input landed in #1467) or it is straight
    /// client-bug input. Sentry cannot help — we can neither retro-fix
    /// already-shipped installs nor learn anything from the noise.
    ///
    /// Logged structurally with the body redacted: these messages embed
    /// caller-supplied param names and, for the `invalid params: …` shape, can
    /// carry deserialized values.
    ParamValidation,
    /// Session-expired bubbles up as an "error" but is an expected boundary
    /// condition (the auth handler clears the local token and the UI
    /// re-auths). Its messages are a small set of fixed strings with no
    /// caller-supplied content, so the full text is safe to log.
    SessionExpired,
    /// A `/teams/me/usage` probe that the failure-backoff in `team::ops`
    /// short-circuited within its window — i.e. an already-reported repeat.
    /// The first failure of the streak already hit the backend and reported
    /// normally; demoting the repeats is the flood control GH #4153 asks for
    /// (backpressure, not silent drop).
    ///
    /// The internal demotion marker must never reach the RPC client as the
    /// error message (CodeRabbit on #4153), so the handler replaces it with
    /// [`USAGE_BACKOFF_CLIENT_MESSAGE`].
    UsageProbeBackoff,
    /// A downstream call (backend_api / integrations / provider) already
    /// demoted the underlying transient failure to a warn. Re-reporting at
    /// error level would re-create the Sentry noise the lower-layer demote was
    /// meant to avoid (#8Z, #93, #8W, #96).
    ///
    /// The message is upstream-derived (backend / provider response) and can
    /// carry URL fragments, query params, or provider error text that
    /// includes tokens, so it is logged only after `sanitize_api_error`.
    TransientDownstream,
    /// An unrecognised RPC method is a transport-boundary mismatch (infra
    /// probe traffic, or a client on a different release than the running
    /// core), not an actionable core defect (#3567).
    ///
    /// Known external probes (`probe == true`) never become real methods, so
    /// they are debug-only and never reach Sentry. Any other unknown method is
    /// still recorded for triage, at warn severity (captured, no page).
    UnknownMethod { probe: bool },
    /// The caller submitted an ingest payload that does not match the
    /// canonicaliser schema for its `source_kind` (#5169). The handler already
    /// returned a precise error naming the missing or malformed field, and no
    /// core-side change can fix a producer sending the wrong shape. Still
    /// captured for triage (a spike means a producer regressed), but at warn
    /// severity so it does not page.
    InvalidIngestPayload,
    /// Everything else: reported through
    /// `observability::report_error_or_expected`.
    Unexpected,
}

/// What the caller sees instead of the usage-probe backoff marker.
#[cfg(feature = "http-server")]
pub(super) const USAGE_BACKOFF_CLIENT_MESSAGE: &str =
    "Usage temporarily unavailable — the last fetch failed and is backing off; \
     it will refresh shortly.";

/// Classify a failed call's display message. `expected_user_state` is the
/// flag from the controller's structured envelope, when it emitted one.
///
/// The order is significant: the first matching rule wins, exactly as the
/// handler's `if`/`else` chain did before it was extracted.
#[cfg(feature = "http-server")]
pub(super) fn classify_failure(message: &str, expected_user_state: bool) -> FailureDisposition {
    use crate::core::observability;

    if expected_user_state {
        FailureDisposition::ExpectedUserState
    } else if is_wallet_not_configured_error(message) {
        FailureDisposition::WalletNotConfigured
    } else if crate::rpc::is_param_validation_error(message) {
        FailureDisposition::ParamValidation
    } else if is_session_expired_error(message) {
        FailureDisposition::SessionExpired
    } else if observability::is_suppressed_usage_probe_backoff(message) {
        FailureDisposition::UsageProbeBackoff
    } else if observability::is_transient_message_failure(message) {
        FailureDisposition::TransientDownstream
    } else if let Some(unknown_method) = crate::core::dispatch::unknown_method_name(message) {
        FailureDisposition::UnknownMethod {
            probe: crate::core::dispatch::is_known_probe_method(unknown_method),
        }
    } else if crate::memory::tree::tree::rpc::is_invalid_ingest_payload_message(message) {
        FailureDisposition::InvalidIngestPayload
    } else {
        FailureDisposition::Unexpected
    }
}

#[cfg(test)]
#[path = "classify_tests.rs"]
mod tests;
