//! Unit tests for the MCP clients RPC handlers.
//!
//! The operations themselves are covered in `tinymcp`. What this layer adds is
//! the RPC shape, which the end-to-end suites exercise against a live process,
//! and how a refusal reads to the caller, which a host over a temp workspace
//! can pin without connecting anything.

use super::*;

#[test]
fn a_blank_identifier_is_refused_with_the_field_name() {
    // The frontend surfaces this text, so it has to name what was missing.
    let error = require("   ", "server_id").expect_err("a blank identifier");
    assert_eq!(error, "server_id must not be empty");
}

#[test]
fn an_identifier_is_trimmed_before_it_is_used() {
    assert_eq!(require("  srv-1  ", "server_id").unwrap(), "srv-1");
}

// ── list_tools refusals (#6313) ─────────────────────────────────────────────
//
// A real host over a temp workspace. `apply_config_doc` writes the install
// store and dials nothing, so an installed server stays disconnected.

const QUALIFIED: &str = "io.github.ryudi84/uuid";

fn temp_config(workspace: &tempfile::TempDir) -> Config {
    Config {
        workspace_dir: workspace.path().join("workspace"),
        action_dir: workspace.path().join("workspace"),
        config_path: workspace.path().join("config.toml"),
        ..Default::default()
    }
}

/// Installs one stdio server under [`QUALIFIED`] and returns its `server_id`.
async fn install_one(config: &Config) -> String {
    let registry = resolve(config).unwrap();
    let doc = json!({ "mcpServers": { QUALIFIED: { "command": "uuid-mcp" } } });
    registry.dynamic().apply_config_doc(&doc).await.unwrap();
    let installs = registry.dynamic().status().await.unwrap();
    assert_eq!(installs.len(), 1, "{installs:?}");
    assert_ne!(
        installs[0].server_id, QUALIFIED,
        "fixture: the install's id must differ from its registry name"
    );
    installs[0].server_id.clone()
}

#[tokio::test]
async fn agent_refusal_names_the_connect_tool_on_its_belt() {
    let workspace = tempfile::tempdir().unwrap();
    let config = temp_config(&workspace);
    let server_id = install_one(&config).await;

    let error = mcp_clients_list_tools(&config, server_id.clone(), Caller::Agent)
        .await
        .expect_err("a disconnected server has no tools to list");
    assert!(error.contains("mcp_registry_connect"), "{error}");
    assert!(!error.contains("mcp_clients_connect"), "{error}");
    assert!(error.contains(&server_id), "{error}");
}

#[tokio::test]
async fn rpc_refusal_still_names_the_rpc_method() {
    let workspace = tempfile::tempdir().unwrap();
    let config = temp_config(&workspace);
    let server_id = install_one(&config).await;

    let error = mcp_clients_list_tools(&config, server_id, Caller::Rpc)
        .await
        .expect_err("a disconnected server has no tools to list");
    assert!(error.contains("mcp_clients_connect"), "{error}");
    assert!(!error.contains("mcp_registry_connect"), "{error}");
}

#[tokio::test]
async fn a_qualified_name_resolves_to_its_install() {
    let workspace = tempfile::tempdir().unwrap();
    let config = temp_config(&workspace);
    let server_id = install_one(&config).await;

    // Resolved, then refused for the install it names — in terms of that
    // install's server_id, not the registry name the caller passed.
    let error = mcp_clients_list_tools(&config, QUALIFIED.to_string(), Caller::Agent)
        .await
        .expect_err("the resolved server is not connected");
    assert!(
        error.starts_with(&format!("server_id={server_id} is disconnected")),
        "{error}"
    );
}

#[tokio::test]
async fn an_uninstalled_id_is_not_reported_as_not_connected() {
    let workspace = tempfile::tempdir().unwrap();
    let config = temp_config(&workspace);
    install_one(&config).await;

    let error = mcp_clients_list_tools(&config, "io.github.other/absent".into(), Caller::Agent)
        .await
        .expect_err("nothing is installed under that name");
    assert!(!error.contains("not connected"), "{error}");
    assert!(error.contains("no installed MCP server"), "{error}");
    assert!(error.contains("mcp_registry_status"), "{error}");
}

#[test]
fn a_registry_name_installed_twice_is_refused_with_both_ids() {
    let install = |server_id: &str| tinymcp::ConnStatus {
        server_id: server_id.into(),
        qualified_name: QUALIFIED.into(),
        display_name: "uuid".into(),
        status: tinymcp::ServerStatus::Disconnected,
        tool_count: 0,
        last_error: None,
        auth_hint: None,
    };
    let installs = [install("srv-a"), install("srv-b")];
    let NotConnected::Refused(message) = explain_not_connected(QUALIFIED, &installs, Caller::Agent)
    else {
        panic!("an ambiguous name must not resolve");
    };
    assert!(message.contains("srv-a, srv-b"), "{message}");
}
