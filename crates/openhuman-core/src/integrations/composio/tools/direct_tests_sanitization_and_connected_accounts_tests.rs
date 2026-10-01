use super::*;

#[test]
fn sanitize_error_message_truncates_long_messages() {
    let long_msg = "x".repeat(500);
    let sanitized = sanitize_error_message(&long_msg);
    assert!(
        sanitized.chars().count() <= 243,
        "should be at most 240 chars + '...'"
    );
    assert!(
        sanitized.ends_with("..."),
        "truncated message should end with '...'"
    );
}

#[test]
fn sanitize_error_message_does_not_truncate_short_messages() {
    let short = "Something went wrong";
    let sanitized = sanitize_error_message(short);
    assert_eq!(sanitized, short);
}

#[test]
fn sanitize_error_message_replaces_all_sensitive_variants() {
    // camelCase variants
    let msg = "Error for connectedAccountId and entityId and userId";
    let sanitized = sanitize_error_message(msg);
    assert!(
        !sanitized.contains("connectedAccountId"),
        "camelCase connectedAccountId should be redacted"
    );
    assert!(
        !sanitized.contains("entityId"),
        "camelCase entityId should be redacted"
    );
    assert!(
        !sanitized.contains("userId"),
        "camelCase userId should be redacted"
    );
}
