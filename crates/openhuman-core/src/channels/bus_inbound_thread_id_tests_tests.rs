use super::delivery::channel_message_body_with_idempotency;
use super::progressive_ui::channel_supports_progressive_ui;
use serde_json::json;

#[test]
fn progressive_ui_is_an_allowlist_failing_safe_for_unknown_channels() {
    // Only edit+delete-capable providers opt in. Telegram supports both;
    // everything else (Discord's stub delete / 404 edits, and any new or
    // unknown adapter) is suppressed so the "💭" spam can't reappear.
    assert!(channel_supports_progressive_ui("telegram"));
    assert!(channel_supports_progressive_ui("tg"));
    // Inbound channels arrive provider-prefixed — the prefix must still match.
    assert!(channel_supports_progressive_ui("tg:12345"));
    assert!(!channel_supports_progressive_ui("discord"));
    assert!(!channel_supports_progressive_ui("discord:guild-1"));
    // Unknown/new adapters fail safe (allowlist, not denylist).
    assert!(!channel_supports_progressive_ui("slack"));
    assert!(!channel_supports_progressive_ui("whatsapp:123"));
}

#[test]
fn channel_message_body_adds_deterministic_idempotency_key() {
    let left = channel_message_body_with_idempotency(
        "telegram",
        json!({ "text": "hello", "threadId": "topic-1" }),
    );
    let right = channel_message_body_with_idempotency(
        "telegram",
        json!({ "threadId": "topic-1", "text": "hello" }),
    );

    assert_eq!(left["text"], "hello");
    assert_eq!(left["threadId"], "topic-1");
    assert_eq!(left["idempotencyKey"], right["idempotencyKey"]);
    assert!(left["idempotencyKey"]
        .as_str()
        .expect("idempotency key")
        .starts_with("legacy-send:telegram:"));
}

#[test]
fn channel_message_body_preserves_caller_idempotency_key() {
    let body = channel_message_body_with_idempotency(
        "discord",
        json!({ "text": "hello", "idempotencyKey": "caller-key" }),
    );

    assert_eq!(body["idempotencyKey"], "caller-key");
}
