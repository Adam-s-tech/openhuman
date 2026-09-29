use super::*;
use tinytools::ToolScope;

fn cfg() -> Arc<Config> {
    Arc::new(Config::default())
}

#[test]
fn names_and_levels() {
    assert_eq!(
        McpRegistrySearchTool::new(cfg()).name(),
        "mcp_registry_search"
    );
    assert_eq!(
        McpRegistrySearchTool::new(cfg()).permission_level(),
        PermissionLevel::ReadOnly
    );
    assert_eq!(
        McpRegistryConnectTool::new(cfg()).permission_level(),
        PermissionLevel::Execute
    );
    assert_eq!(
        McpRegistryToolCallTool::new(cfg()).permission_level(),
        PermissionLevel::Execute
    );
    // Discovery tool: read-only, names match.
    assert_eq!(
        McpRegistryListToolsTool::new(cfg()).name(),
        "mcp_registry_list_tools"
    );
    assert_eq!(
        McpRegistryListToolsTool::new(cfg()).permission_level(),
        PermissionLevel::ReadOnly
    );
    assert_eq!(
        McpRegistryUninstallTool::new(cfg()).permission_level(),
        PermissionLevel::Write
    );
    assert_eq!(McpRegistrySearchTool::new(cfg()).scope(), ToolScope::All);
}

#[tokio::test]
async fn get_requires_qualified_name() {
    let err = McpRegistryGetTool::new(cfg())
        .execute(json!({}))
        .await
        .expect_err("missing qualified_name");
    assert!(err.to_string().contains("qualified_name"));
}

#[tokio::test]
async fn list_tools_requires_server_id() {
    let err = McpRegistryListToolsTool::new(cfg())
        .execute(json!({}))
        .await
        .expect_err("missing server_id");
    assert!(err.to_string().contains("server_id"));
}

#[tokio::test]
async fn list_tools_errors_for_unconnected_server() {
    // A server_id that is not in the live connection map surfaces a
    // "connect first" hint rather than an empty success.
    let err = McpRegistryListToolsTool::new(cfg())
        .execute(json!({ "server_id": "definitely-not-connected-uuid" }))
        .await
        .expect_err("unconnected server must error");
    assert!(
        err.to_string().contains("not connected"),
        "expected connect-first hint, got: {err}"
    );
}

#[test]
fn zz_dump_goldens_tmp() {
    let tools: Vec<Box<dyn Tool>> = vec![
        Box::new(McpRegistrySearchTool::new(cfg())),
        Box::new(McpRegistryGetTool::new(cfg())),
        Box::new(McpRegistryInstalledListTool::new(cfg())),
        Box::new(McpRegistryStatusTool::new(cfg())),
        Box::new(McpRegistryListToolsTool::new(cfg())),
        Box::new(McpRegistryConnectTool::new(cfg())),
        Box::new(McpRegistryDisconnectTool::new(cfg())),
        Box::new(McpRegistryToolCallTool::new(cfg())),
        Box::new(McpRegistryUninstallTool::new(cfg())),
    ];
    let mut out = Vec::new();
    for t in &tools {
        out.push(json!({
            "name": t.name(),
            "description": t.description(),
            "schema_text": serde_json::to_string(&t.parameters_schema()).unwrap(),
            "permission": format!("{:?}", t.permission_level()),
            "exposure": format!("{:?}", t.exposure()),
            "concurrency_safe": t.is_concurrency_safe(&json!({})),
            "external_effect": t.external_effect(),
            "category": format!("{:?}", t.category()),
            "family": t.family(),
        }));
    }
    std::fs::write("/private/tmp/claude-501/-Users-enamakel-work-workflow-openhuman-openhuman/696b53c5-72b2-4872-8939-4b861de0dff7/scratchpad/registry_goldens.json", serde_json::to_string_pretty(&out).unwrap()).unwrap();
}
