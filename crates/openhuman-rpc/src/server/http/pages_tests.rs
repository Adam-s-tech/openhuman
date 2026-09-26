use super::escape_html;

#[test]
fn escape_html_escapes_all_special_chars() {
    let raw = r#"<script>alert("x&y'z")</script>"#;
    let escaped = escape_html(raw);
    assert!(!escaped.contains('<'));
    assert!(!escaped.contains('>'));
    assert!(!escaped.contains('"'));
    assert!(!escaped.contains('\''));
    assert!(escaped.contains("&lt;"));
    assert!(escaped.contains("&gt;"));
    assert!(escaped.contains("&quot;"));
    assert!(escaped.contains("&#x27;"));
    // `&` must be escaped first so later substitutions don't double-encode.
    assert!(escaped.contains("&amp;y"));
}

#[test]
fn escape_html_is_noop_for_safe_text() {
    assert_eq!(escape_html("safe text 123"), "safe text 123");
    assert_eq!(escape_html(""), "");
}
