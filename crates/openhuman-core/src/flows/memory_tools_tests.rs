use super::*;

// ── flow_namespace / FLOW_MEMORY_NAMESPACE_PREFIX ───────────────
// (relocated from `flows::mod` — see that module's re-export comment)

#[test]
fn flow_namespace_uses_the_shared_root_prefix() {
    assert_eq!(flow_namespace("abc-123"), "flow_abc-123");
    assert!(flow_namespace("abc-123").starts_with(FLOW_MEMORY_NAMESPACE_PREFIX));
}

#[test]
fn flow_namespace_is_distinct_per_flow() {
    assert_ne!(flow_namespace("a"), flow_namespace("b"));
}

// ── FlowMemoryRecallTool ────────────────────────────────────────

#[test]
fn recall_name_and_schema() {
    let tool = FlowMemoryRecallTool::new();
    assert_eq!(tool.name(), "flow_memory_recall");
    let schema = tool.parameters_schema();
    assert!(schema["properties"]["query"].is_object());
    assert!(schema["properties"]["flow_id"].is_object());
    assert!(schema["properties"]["scope"].is_object());
}
// ── FlowMemoryRememberTool ──────────────────────────────────────

#[test]
fn remember_name_and_schema() {
    let tool = FlowMemoryRememberTool::new(Arc::new(SecurityPolicy::default()));
    assert_eq!(tool.name(), "flow_memory_remember");
    let schema = tool.parameters_schema();
    assert!(schema["properties"]["flow_id"].is_object());
    assert!(schema["properties"]["key"].is_object());
    assert!(schema["properties"]["content"].is_object());
    // No `namespace` parameter exists — the security invariant that a
    // flow can never target another namespace.
    assert!(schema["properties"]["namespace"].is_null());
    assert_eq!(tool.permission_level(), PermissionLevel::Write);
}
