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

/// The identity a registry tool presents to the model, captured from the
/// hand-written tools before their specs moved to `tinymcp_bus::agent_tools`.
///
/// Name, description and schema are prompt-cache and transcript identity: a
/// byte of drift invalidates every cached prefix and changes what a resumed
/// session replays. The schema is compared as serialized text, not as a
/// `Value`, so a key-order change is caught too.
struct Golden {
    tool: Box<dyn Tool>,
    name: &'static str,
    description: &'static str,
    schema: &'static str,
    permission: PermissionLevel,
    exposure: tinytools::ToolExposure,
    concurrency_safe: bool,
}

fn goldens() -> Vec<Golden> {
    vec![
        Golden {
            tool: Box::new(McpRegistrySearchTool::new(cfg())),
            name: r#"mcp_registry_search"#,
            description: r#"Search the MCP server registry catalog by `query`, optionally filtered by `transport` ("stdio" | "hosted" | "all"), paginated by `page` / `page_size`. Use to discover installable MCP servers."#,
            schema: r#"{"properties":{"page":{"minimum":1,"type":"integer"},"page_size":{"minimum":1,"type":"integer"},"query":{"type":"string"},"transport":{"enum":["stdio","hosted","all"],"type":"string"}},"type":"object"}"#,
            permission: PermissionLevel::ReadOnly,
            exposure: tinytools::ToolExposure::Deferred,
            concurrency_safe: true,
        },
        Golden {
            tool: Box::new(McpRegistryGetTool::new(cfg())),
            name: r#"mcp_registry_get"#,
            description: r#"Get one MCP registry server's detail by `qualified_name`."#,
            schema: r#"{"properties":{"qualified_name":{"type":"string"}},"required":["qualified_name"],"type":"object"}"#,
            permission: PermissionLevel::ReadOnly,
            exposure: tinytools::ToolExposure::Deferred,
            concurrency_safe: true,
        },
        Golden {
            tool: Box::new(McpRegistryInstalledListTool::new(cfg())),
            name: r#"mcp_registry_installed_list"#,
            description: r#"List the MCP servers currently installed for this user."#,
            schema: r#"{"properties":{},"type":"object"}"#,
            permission: PermissionLevel::ReadOnly,
            exposure: tinytools::ToolExposure::Deferred,
            concurrency_safe: true,
        },
        Golden {
            tool: Box::new(McpRegistryStatusTool::new(cfg())),
            name: r#"mcp_registry_status"#,
            description: r#"Report the connection status of installed MCP servers."#,
            schema: r#"{"properties":{},"type":"object"}"#,
            permission: PermissionLevel::ReadOnly,
            exposure: tinytools::ToolExposure::Direct,
            concurrency_safe: true,
        },
        Golden {
            tool: Box::new(McpRegistryListToolsTool::new(cfg())),
            name: r#"mcp_registry_list_tools"#,
            description: r#"List the tools (name, description, input schema) exposed by a connected MCP server, given its `server_id`. Use this to discover what a connected server can do before calling `mcp_registry_tool_call`. The server must already be connected (see `mcp_registry_status` / `mcp_registry_connect`)."#,
            schema: r#"{"properties":{"server_id":{"type":"string"}},"required":["server_id"],"type":"object"}"#,
            permission: PermissionLevel::ReadOnly,
            exposure: tinytools::ToolExposure::Direct,
            concurrency_safe: true,
        },
        Golden {
            tool: Box::new(McpRegistryConnectTool::new(cfg())),
            name: r#"mcp_registry_connect"#,
            description: r#"Connect (spawn + handshake) an installed MCP server by `server_id`, returning its tools."#,
            schema: r#"{"properties":{"server_id":{"type":"string"}},"required":["server_id"],"type":"object"}"#,
            permission: PermissionLevel::Execute,
            exposure: tinytools::ToolExposure::Direct,
            concurrency_safe: false,
        },
        Golden {
            tool: Box::new(McpRegistryDisconnectTool::new(cfg())),
            name: r#"mcp_registry_disconnect"#,
            description: r#"Disconnect (stop) a connected MCP server by `server_id`."#,
            schema: r#"{"properties":{"server_id":{"type":"string"}},"required":["server_id"],"type":"object"}"#,
            permission: PermissionLevel::Execute,
            exposure: tinytools::ToolExposure::Direct,
            concurrency_safe: false,
        },
        Golden {
            tool: Box::new(McpRegistryToolCallTool::new(cfg())),
            name: r#"mcp_registry_tool_call"#,
            description: r#"Invoke a tool on a connected MCP server: `server_id` + `tool_name` + `arguments` object."#,
            schema: r#"{"properties":{"arguments":{"type":"object"},"server_id":{"type":"string"},"tool_name":{"type":"string"}},"required":["server_id","tool_name"],"type":"object"}"#,
            permission: PermissionLevel::Execute,
            exposure: tinytools::ToolExposure::Direct,
            concurrency_safe: false,
        },
        Golden {
            tool: Box::new(McpRegistryUninstallTool::new(cfg())),
            name: r#"mcp_registry_uninstall"#,
            description: r#"Uninstall an installed MCP server by `server_id`. Default-OFF (opt-in)."#,
            schema: r#"{"properties":{"server_id":{"type":"string"}},"required":["server_id"],"type":"object"}"#,
            permission: PermissionLevel::Write,
            exposure: tinytools::ToolExposure::Direct,
            concurrency_safe: false,
        },
    ]
}

#[test]
fn registry_tools_present_the_exact_identity_they_always_had() {
    for golden in goldens() {
        let tool = &golden.tool;
        assert_eq!(tool.name(), golden.name);
        assert_eq!(tool.description(), golden.description, "{}", golden.name);
        assert_eq!(
            serde_json::to_string(&tool.parameters_schema()).unwrap(),
            golden.schema,
            "{}",
            golden.name
        );
        assert_eq!(
            tool.permission_level(),
            golden.permission,
            "{}",
            golden.name
        );
        assert_eq!(tool.exposure(), golden.exposure, "{}", golden.name);
        assert_eq!(
            tool.is_concurrency_safe(&json!({})),
            golden.concurrency_safe,
            "{}",
            golden.name
        );
        assert!(!tool.external_effect(), "{}", golden.name);
        assert_eq!(tool.family(), None, "{}", golden.name);
    }
}
