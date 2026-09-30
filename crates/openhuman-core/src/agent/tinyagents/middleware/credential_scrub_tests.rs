//! The browser confirmation-token exemption: the one piece of credential
//! scrubbing policy the host owns. The scrubber, its notice and their tests
//! live in `tinyagents-harness` (`middleware::library::credential_scrub`).

use super::*;

#[test]
fn browser_confirmation_token_survives_without_exempting_page_credentials() {
    let pending = serde_json::json!({
        "status": "NeedsConfirmation",
        "pending": {
            "action": {"action": "click", "target": {"kind": "ref", "value": "e33"}},
            "token": "00000000-0000-4000-8000-000000000123"
        },
        "page": {
            "api_key": "sk-abcdefghijklmnopqrstuvwxyz123456",
            "text": "token=page-secret-value"
        }
    });
    let content = pending.to_string();
    let (scrubbed, count) =
        scrub_with_notice_for_tool("browser", &content).expect("page credential is still redacted");
    assert_eq!(count, 2);
    assert!(scrubbed.contains("00000000-0000-4000-8000-000000000123"));
    assert!(!scrubbed.contains("abcdefghijklmnopqrstuvwxyz123456"));
    assert!(!scrubbed.contains("page-secret-value"));
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(scrubbed.split("\n\n").next().unwrap()).unwrap()
            ["pending"]["token"],
        pending["pending"]["token"]
    );

    let token_only = serde_json::json!({
        "status": "NeedsConfirmation",
        "pending": {"token": "00000000-0000-4000-8000-000000000123"}
    });
    assert!(scrub_with_notice_for_tool("browser", &token_only.to_string()).is_none());
}
