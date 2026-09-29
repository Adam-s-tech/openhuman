use super::*;
use crate::config::{McpAuthConfig, McpServerConfig};
use serde_json::json;

const SECRET: &str = "sk-live-configured-secret-123";

struct Server;

impl wiremock::Respond for Server {
    fn respond(&self, request: &wiremock::Request) -> wiremock::ResponseTemplate {
        let body: Value = serde_json::from_slice(&request.body).unwrap_or_default();
        let result = match body["method"].as_str().unwrap_or_default() {
            "initialize" => json!({
                "protocolVersion": tinymcp_bus::LATEST_PROTOCOL_VERSION,
                "capabilities": { "tools": {} },
                "serverInfo": { "name": "goals", "version": "1.0.0" },
            }),
            "notifications/initialized" => return wiremock::ResponseTemplate::new(202),
            "tools/list" => json!({ "tools": [
                {
                    "name": "readGoals",
                    "description": "Read the goals on a list",
                    "inputSchema": { "type": "object", "properties": { "list": { "type": "string" } } },
                },
                { "name": "archive", "description": "Archive a goal" },
            ]}),
            "tools/call" => json!({
                "content": [{ "type": "text", "text": format!("goals for {SECRET}") }],
            }),
            _ => json!({}),
        };
        wiremock::ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": body["id"].clone(),
            "result": result,
        }))
    }
}

async fn server() -> wiremock::MockServer {
    let server = wiremock::MockServer::start().await;
    wiremock::Mock::given(wiremock::matchers::method("POST"))
        .respond_with(Server)
        .mount(&server)
        .await;
    server
}

fn config(workspace: &std::path::Path, endpoint: &str, customize: impl FnOnce(&mut McpServerConfig)) -> Config {
    let mut config = Config {
        workspace_dir: workspace.join("workspace"),
        action_dir: workspace.join("workspace"),
        config_path: workspace.join("config.toml"),
        ..Default::default()
    };
    config.gitbooks.enabled = false;
    let mut server = McpServerConfig {
        name: "ticktick".into(),
        endpoint: endpoint.into(),
        auth: McpAuthConfig::BearerToken {
            token: SECRET.into(),
        },
        ..Default::default()
    };
    customize(&mut server);
    config.mcp_client.servers.push(server);
    config
}

async fn warmed(config: &Config) -> Arc<McpServerRegistry> {
    let registry = Arc::new(crate::mcp::host::static_registry(config));
    let host = crate::mcp::host::for_config(config).expect("host");
    for (name, outcome) in registry.refresh_tool_cache(host.dynamic().store()).await {
        outcome.unwrap_or_else(|error| panic!("{name}: {error}"));
    }
    registry
}

fn security() -> Arc<SecurityPolicy> {
    Arc::new(SecurityPolicy::default())
}

#[tokio::test]
async fn cached_tools_become_deferred_mcp_server_tool_names() {
    let mock = server().await;
    let dir = tempfile::tempdir().unwrap();
    let config = config(dir.path(), &format!("{}/mcp", mock.uri()), |_| {});
    let registry = warmed(&config).await;

    let tools = configured_server_tools(&config, &registry, &security(), &HashSet::new());
    let names: Vec<&str> = tools.iter().map(|tool| tool.name()).collect();
    assert_eq!(names, ["mcp_ticktick_archive", "mcp_ticktick_read_goals"]);
    assert!(tools
        .iter()
        .all(|tool| tool.exposure() == ToolExposure::Deferred));
    assert_eq!(tools[1].family(), Some("ticktick"));
    assert_eq!(tools[1].permission_level(), PermissionLevel::Execute);
    assert!(tools[1].external_effect());
}

#[tokio::test]
async fn a_call_reaches_the_server_and_its_output_is_scrubbed() {
    let mock = server().await;
    let dir = tempfile::tempdir().unwrap();
    let config = config(dir.path(), &format!("{}/mcp", mock.uri()), |_| {});
    let registry = warmed(&config).await;
    let tools = configured_server_tools(&config, &registry, &security(), &HashSet::new());

    let read = tools
        .iter()
        .find(|tool| tool.name() == "mcp_ticktick_read_goals")
        .unwrap();
    let result = read.execute(json!({ "list": "work" })).await.unwrap();
    assert!(!result.is_error, "{}", result.text());
    assert_eq!(result.text(), "goals for [redacted]");
}

#[tokio::test]
async fn expose_direct_and_direct_tools_are_honoured() {
    let mock = server().await;
    let dir = tempfile::tempdir().unwrap();
    let endpoint = format!("{}/mcp", mock.uri());

    let config_direct = config(dir.path(), &endpoint, |server| {
        server.expose = McpToolExposure::Direct;
    });
    let registry = warmed(&config_direct).await;
    let tools = configured_server_tools(&config_direct, &registry, &security(), &HashSet::new());
    assert!(tools.iter().all(|tool| tool.exposure() == ToolExposure::Direct));

    let config_pinned = config(dir.path(), &endpoint, |server| {
        server.direct_tools = vec!["readGoals".into()];
    });
    let registry = warmed(&config_pinned).await;
    let tools = configured_server_tools(&config_pinned, &registry, &security(), &HashSet::new());
    let exposure = |name: &str| {
        tools
            .iter()
            .find(|tool| tool.name() == name)
            .unwrap()
            .exposure()
    };
    assert_eq!(exposure("mcp_ticktick_read_goals"), ToolExposure::Direct);
    assert_eq!(exposure("mcp_ticktick_archive"), ToolExposure::Deferred);
}

#[tokio::test]
async fn a_cold_cache_yields_no_tools_and_reserved_names_are_kept() {
    let mock = server().await;
    let dir = tempfile::tempdir().unwrap();
    let config = config(dir.path(), &format!("{}/mcp", mock.uri()), |_| {});

    let cold = Arc::new(crate::mcp::host::static_registry(&config));
    assert!(configured_server_tools(&config, &cold, &security(), &HashSet::new()).is_empty());

    let registry = warmed(&config).await;
    let reserved: HashSet<String> = ["mcp_ticktick_archive".to_string()].into_iter().collect();
    let tools = configured_server_tools(&config, &registry, &security(), &reserved);
    let names: Vec<&str> = tools.iter().map(|tool| tool.name()).collect();
    assert_eq!(names, ["mcp_ticktick_read_goals"]);
}
