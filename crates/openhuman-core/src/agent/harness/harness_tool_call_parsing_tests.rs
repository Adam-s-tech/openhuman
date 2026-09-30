use super::*;

#[test]
fn test_scrub_credentials() {
    let input = "API_KEY=sk-1234567890abcdef; token: 1234567890; password=\"secret123456\"";
    let scrubbed = scrub_credentials(input);
    assert!(scrubbed.contains("API_KEY=sk-1*[REDACTED]"));
    assert!(scrubbed.contains("token: 1234*[REDACTED]"));
    assert!(scrubbed.contains("password=\"secr*[REDACTED]\""));
    assert!(!scrubbed.contains("abcdef"));
    assert!(!scrubbed.contains("secret123456"));
}

#[test]
fn parse_tool_calls_recovers_cross_alias_closing_tags() {
    let response = r#"<toolcall>
{"name": "shell", "arguments": {"command": "date"}}
</tool_call>"#;

    let (text, calls) = parse_tool_calls(response);
    assert!(text.is_empty());
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].name, "shell");
}

#[test]
fn parse_tool_calls_rejects_raw_tool_json_without_tags() {
    // SECURITY: Raw JSON without explicit wrappers should NOT be parsed
    // This prevents prompt injection attacks where malicious content
    // could include JSON that mimics a tool call.
    let response = r#"Sure, creating the file now.
{"name": "file_write", "arguments": {"path": "hello.py", "content": "print('hello')"}}"#;

    let (text, calls) = parse_tool_calls(response);
    assert!(text.contains("Sure, creating the file now."));
    assert_eq!(
        calls.len(),
        0,
        "Raw JSON without wrappers should not be parsed"
    );
}

#[test]
fn build_tool_instructions_includes_all_tools() {
    use crate::security::SecurityPolicy;
    let security = Arc::new(SecurityPolicy::from_config(
        &crate::config::AutonomyConfig::default(),
        std::path::Path::new("/tmp"),
        std::path::Path::new("/tmp"),
    ));
    let tools = tools::ops::default_tools(security);
    let instructions = build_tool_instructions(&tools);

    assert!(instructions.contains("## Tool Use Protocol"));
    assert!(instructions.contains("<tool_call>"));
    assert!(instructions.contains("shell"));
    assert!(instructions.contains("file_read"));
    assert!(instructions.contains("file_write"));
}

#[test]
fn tools_to_openai_format_produces_valid_schema() {
    use crate::security::SecurityPolicy;
    let security = Arc::new(SecurityPolicy::from_config(
        &crate::config::AutonomyConfig::default(),
        std::path::Path::new("/tmp"),
        std::path::Path::new("/tmp"),
    ));
    let tools = tools::ops::default_tools(security);
    let formatted = tools_to_openai_format(&tools);

    assert!(!formatted.is_empty());
    for tool_json in &formatted {
        assert_eq!(tool_json["type"], "function");
        assert!(tool_json["function"]["name"].is_string());
        assert!(tool_json["function"]["description"].is_string());
        assert!(!tool_json["function"]["name"].as_str().unwrap().is_empty());
    }
    // Verify known tools are present
    let names: Vec<&str> = formatted
        .iter()
        .filter_map(|t| t["function"]["name"].as_str())
        .collect();
    assert!(names.contains(&"shell"));
    assert!(names.contains(&"file_read"));
}

// ═══════════════════════════════════════════════════════════════════════
// Recovery Tests - Tool Call Parsing Edge Cases
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn parse_tool_calls_handles_empty_tool_result() {
    // Recovery: Empty tool_result tag should be handled gracefully
    let response = r#"I'll run that command.
<tool_result name="shell">

</tool_result>
Done."#;
    let (text, calls) = parse_tool_calls(response);
    assert!(text.contains("Done."));
    assert!(calls.is_empty());
}

#[test]
fn parse_tool_calls_handles_empty_tool_calls_array() {
    // Recovery: Empty tool_calls array returns original response (no tool parsing)
    let response = r#"{"content": "Hello", "tool_calls": []}"#;
    let (text, calls) = parse_tool_calls(response);
    // When tool_calls is empty, the entire JSON is returned as text
    assert!(text.contains("Hello"));
    assert!(calls.is_empty());
}
