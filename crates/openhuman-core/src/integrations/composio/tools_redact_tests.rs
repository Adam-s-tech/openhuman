//! Regression coverage for `redact.rs` gaps flagged by review on PR #6689.

use serde_json::json;

use crate::config::{Config, ComposioHostCredential};
use crate::integrations::composio::tools::redact_composio_outcome;

const SHORT_KEY: &str = "abcdefgh";
const LONG_KEY: &str = "abcdefghi";

fn config_with_overlapping_secrets() -> Config {
    let tmp = tempfile::tempdir().expect("tempdir for config_with_overlapping_secrets");
    let mut config = Config::default();
    config.config_path = tmp.path().join("config.toml");
    config.composio.api_key = Some(LONG_KEY.to_string());
    config
        .composio
        .pin_host_credential(ComposioHostCredential::direct(SHORT_KEY));
    std::mem::forget(tmp);
    config
}

/// Before the fix, `composio_secrets` lexicographically sorted
/// `[SHORT_KEY, LONG_KEY]` (`SHORT_KEY` sorts first since it's a prefix)
/// and redacted in that order, replacing `SHORT_KEY` everywhere first —
/// including inside `LONG_KEY` — and leaving the trailing `i` behind
/// instead of a clean `[REDACTED]`.
#[test]
fn overlapping_secrets_redact_longest_first() {
    let config = config_with_overlapping_secrets();
    let outcome = redact_composio_outcome(
        &config,
        Ok(tinytools::ToolResult::success(format!(
            "token {LONG_KEY} in transit"
        ))),
    )
    .unwrap();
    let text = outcome
        .content
        .iter()
        .filter_map(|c| match c {
            tinytools::ToolContent::Text { text } => Some(text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ");
    assert!(!text.contains(LONG_KEY), "{text}");
    assert!(!text.contains(SHORT_KEY), "{text}");
    assert_eq!(text, "token [REDACTED] in transit");
}

/// `redact_value` used to walk only object *values*; a secret that
/// happened to be used as a JSON object key survived untouched.
#[test]
fn secret_used_as_a_json_object_key_is_redacted() {
    let config = config_with_overlapping_secrets();
    let outcome = redact_composio_outcome(
        &config,
        Ok(tinytools::ToolResult::json(
            json!({ format!("token_{LONG_KEY}"): "value" }),
        )),
    )
    .unwrap();
    let serialized = serde_json::to_string(&outcome).unwrap();
    assert!(!serialized.contains(LONG_KEY), "{serialized}");
    assert!(serialized.contains("[REDACTED]"), "{serialized}");
}
