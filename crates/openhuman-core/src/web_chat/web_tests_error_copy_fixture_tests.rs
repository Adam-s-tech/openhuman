//! GENERATOR (temporary)
use super::*;

pub(super) const CASES: &[(&str, &str)] = &[
    ("codex_expired", "Codex authentication token is expired; sign in again"),
    ("session_expired", "SESSION_EXPIRED"),
    ("action_budget", "Action blocked: rate limit exceeded for tool curl"),
    ("max_iterations", "Agent exceeded maximum tool iterations (10)"),
    ("turn_timeout_marker", "openhuman_turn_wall_clock_timeout: agent turn exceeded its 600s wall-clock budget"),
    ("turn_timeout_harness", "run timed out: tool call for run `x` exceeded its remaining wall-clock budget (5 ms)"),
    ("empty_response", "The model returned an empty response. Please try again."),
    ("chat_template", "openai API error (400): Jinja Exception: No user query found in messages."),
    ("rate_limited_transient", "openrouter API error (429 Too Many Requests): Retry-After: 30"),
    ("rate_limited_business", "openai API error (429): plan does not include this model; insufficient balance"),
    ("timeout", "request timed out after 30s"),
    ("auth_error", "openai API error (401 Unauthorized): invalid api key"),
    ("budget_402", "openai API error (402 Payment Required)"),
    ("budget_phrase", "Insufficient budget: please top up"),
    ("budget_phrase_provider_source", "openrouter API error (400): out of credits"),
    ("provider_5xx", "anthropic API error (503 Service Unavailable)"),
    ("context_overflow", "context length exceeded"),
    ("config_rejection", "openai API error (400): model `reasoning-v1` does not exist"),
    ("model_unavailable", "model foo not found on this endpoint"),
    ("model_transient", "the model is temporarily unavailable"),
    ("vision", "provider_capability_error capability=vision does not support vision input"),
    ("malformed_history_byo", "openai API error (400): messages with role 'tool' must be a response to a preceding message with 'tool_calls'"),
    ("request_rejected_4xx", "someprovider API error (422 Unprocessable Entity): bad thing"),
    ("network", "error sending request: connection reset by peer"),
    ("transient_529", "anthropic API error (529): overloaded"),
    ("generic", "kaboom"),
    ("fallback_chain", "All providers/models failed. Attempts: x"),
    ("be_rate_limited", r#"OpenHuman API error (429): {"error":{"errorCode":"RATE_LIMITED","retryAfter":30}}"#),
    ("be_credits", r#"OpenHuman API error (402): {"error":{"errorCode":"USER_INSUFFICIENT_CREDITS"}}"#),
    ("be_upstream", r#"OpenHuman API error (502): {"error":{"errorCode":"UPSTREAM_UNAVAILABLE"}}"#),
    ("be_model_unavailable", r#"OpenHuman API error (503): {"error":{"errorCode":"MODEL_UNAVAILABLE"}}"#),
    ("be_payload", r#"OpenHuman API error (413): {"error":{"errorCode":"PAYLOAD_TOO_LARGE"}}"#),
    ("be_context", r#"OpenHuman API error (400): {"error":{"errorCode":"CONTEXT_LENGTH_EXCEEDED"}}"#),
    ("be_bad_request", r#"OpenHuman API error (400): {"error":{"errorCode":"BAD_REQUEST","message":"x"}}"#),
    ("be_bad_request_malformed", r#"OpenHuman API error (400): {"error":{"errorCode":"BAD_REQUEST","malformed":true}}"#),
    ("be_bad_request_history", r#"OpenHuman API error (400): {"error":{"errorCode":"BAD_REQUEST","message":"messages with role 'tool' must be a response to a preceding message with 'tool_calls'"}}"#),
    ("be_internal", r#"OpenHuman API error (500): {"error":{"errorCode":"INTERNAL_ERROR"}}"#),
];

#[test]
fn dump_cases() {
    let mut out = String::new();
    for (name, input) in CASES {
        let c = classify_inference_error(input);
        out.push_str(&format!(
            "    ({name:?}, {input:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}),\n",
            c.error_type, c.source, c.retryable, c.retry_after_ms, c.provider, c.fallback_available, c.message
        ));
    }
    std::fs::write(concat!(env!("CARGO_MANIFEST_DIR"), "/../../dump.txt"), out).unwrap();
}
