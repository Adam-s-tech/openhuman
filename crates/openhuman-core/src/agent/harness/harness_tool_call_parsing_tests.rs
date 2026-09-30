use super::*;

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
