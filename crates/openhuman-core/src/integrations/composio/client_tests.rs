use super::*;
use crate::config::Config;

use axum::{extract::State, http::StatusCode, routing::get, Json, Router};
use serde_json::json;
use std::sync::atomic::{AtomicUsize, Ordering};

async fn start_mock_backend(app: Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    format!("http://127.0.0.1:{}", addr.port())
}

// ── Route resolution tests (`resolve_composio_route`) ─────────────────
//
// Mirror the branches the spec demands:
//   1. backend mode with a session JWT — Backend variant
//   2. direct mode + stored api key — Direct variant
//   3. direct mode without api key — explicit error
//   4. unknown mode string — explicit error

fn config_with_session_token(tmp: &tempfile::TempDir) -> Config {
    let mut config = Config::default();
    config.config_path = tmp.path().join("config.toml");
    crate::security::credentials::AuthService::from_config(&config)
        .store_provider_token(
            crate::security::credentials::APP_SESSION_PROVIDER,
            crate::security::credentials::DEFAULT_AUTH_PROFILE_NAME,
            "test-token",
            std::collections::HashMap::new(),
            true,
        )
        .expect("store test session token");
    config
}

/// Direct-mode reads are exercised over HTTP through the connector module:
/// `DirectCredential::new_with_v3_base` points the module's `/tools` and
/// `/connected_accounts` GETs at a local axum mock, so we can assert the
/// outbound `tags` filter (repeated query params) and the v3 ->
/// canonical-envelope reshape without touching `backend.composio.dev`. These
/// tests reach the process-global module, so they hold `module_guard`.
fn direct_tool_for_mock(base_v3: String) -> std::sync::Arc<DirectCredential> {
    direct_tool_for_mock_with_key(base_v3, "ck_test_direct")
}

fn direct_tool_for_mock_with_key(
    base_v3: String,
    api_key: &str,
) -> std::sync::Arc<DirectCredential> {
    std::sync::Arc::new(DirectCredential::new_with_v3_base(
        api_key, base_v3,
    ))
}

/// A config that can load the connector module and names no route.
fn module_test_config(tmp: &tempfile::TempDir) -> Config {
    let mut config = Config::default();
    config.config_path = tmp.path().join("config.toml");
    config.workspace_dir = tmp.path().join("workspace");
    config
}

struct DirectAuthFailureGuard {
    key_id: u64,
}

impl DirectAuthFailureGuard {
    fn for_tool(tool: &std::sync::Arc<DirectCredential>) -> Self {
        let key_id = tool.auth_key_fingerprint();
        crate::integrations::composio::direct_auth::reset_direct_auth_failure(key_id);
        Self { key_id }
    }
}

impl Drop for DirectAuthFailureGuard {
    fn drop(&mut self) {
        crate::integrations::composio::direct_auth::reset_direct_auth_failure(self.key_id);
    }
}

#[test]
fn resolve_composio_route_backend_variant_when_mode_default() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let config = config_with_session_token(&tmp);
    let route = resolve_composio_route(&config).expect("backend mode should build");
    assert_eq!(route.mode(), "backend");
    assert!(matches!(route, ComposioRoute::Backend));
}

#[test]
fn resolve_composio_route_backend_empty_mode_falls_back_to_backend() {
    // A literal empty string in TOML should be treated as the default
    // (`"backend"`) rather than an unknown mode error.
    let tmp = tempfile::tempdir().expect("tempdir");
    let mut config = config_with_session_token(&tmp);
    config.composio.mode = String::new();
    let route = resolve_composio_route(&config).expect("empty mode should fall back to backend");
    assert_eq!(route.mode(), "backend");
}

#[test]
fn resolve_composio_route_backend_errors_without_session() {
    // Backend mode requires the app-session JWT — without it the
    // factory must return an explicit error (not silently downgrade).
    let tmp = tempfile::tempdir().expect("tempdir");
    let mut config = crate::config::Config::default();
    config.config_path = tmp.path().join("config.toml");
    let err = resolve_composio_route(&config)
        .err()
        .expect("must error without auth token");
    assert!(
        err.to_string().contains("no backend session token"),
        "unexpected error: {err}"
    );
}

#[test]
fn resolve_composio_route_direct_variant_with_stored_key() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let mut config = crate::config::Config::default();
    config.config_path = tmp.path().join("config.toml");
    config.composio.mode = "direct".into();
    // Persist the key the way the RPC layer would.
    crate::security::credentials::AuthService::from_config(&config)
        .store_provider_token(
            crate::security::credentials::COMPOSIO_DIRECT_PROVIDER,
            crate::security::credentials::DEFAULT_AUTH_PROFILE_NAME,
            "ck_test_key_redacted",
            std::collections::HashMap::new(),
            true,
        )
        .expect("store direct api key");
    let route = resolve_composio_route(&config).expect("direct mode with stored key should build");
    assert_eq!(route.mode(), "direct");
    assert!(matches!(route, ComposioRoute::Direct(_)));
}

#[test]
fn resolve_composio_route_direct_falls_back_to_config_api_key() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let mut config = crate::config::Config::default();
    config.config_path = tmp.path().join("config.toml");
    config.composio.mode = "direct".into();
    // No keychain entry — fall back to the inline config field.
    config.composio.api_key = Some("ck_inline_redacted".into());
    let route = resolve_composio_route(&config)
        .expect("direct mode should accept inline config.api_key when keychain is empty");
    assert_eq!(route.mode(), "direct");
}

#[test]
fn resolve_composio_route_direct_errors_without_key() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let mut config = crate::config::Config::default();
    config.config_path = tmp.path().join("config.toml");
    config.composio.mode = "direct".into();
    let err = resolve_composio_route(&config)
        .err()
        .expect("direct without key must error");
    let msg = err.to_string();
    assert!(
        msg.contains("no api key is configured"),
        "unexpected error: {msg}"
    );
}

#[test]
fn resolve_composio_route_unknown_mode_errors() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let mut config = crate::config::Config::default();
    config.config_path = tmp.path().join("config.toml");
    config.composio.mode = "voyage".into();
    let err = resolve_composio_route(&config)
        .err()
        .expect("unknown mode must error");
    let msg = err.to_string();
    assert!(msg.contains("unknown composio mode"), "got: {msg}");
    assert!(
        msg.contains("voyage"),
        "should echo the invalid value, got: {msg}"
    );
}

// ── Direct-mode credentials helpers ─────────────────────────────────

#[test]
fn store_get_clear_composio_api_key_roundtrip() {
    use crate::security::credentials::{get_composio_api_key, COMPOSIO_DIRECT_PROVIDER};

    let tmp = tempfile::tempdir().expect("tempdir");
    let mut config = crate::config::Config::default();
    config.config_path = tmp.path().join("config.toml");

    // Initially: nothing stored.
    assert_eq!(
        get_composio_api_key(&config).expect("read empty store"),
        None
    );

    // Store under the direct-mode provider slot.
    crate::security::credentials::AuthService::from_config(&config)
        .store_provider_token(
            COMPOSIO_DIRECT_PROVIDER,
            crate::security::credentials::DEFAULT_AUTH_PROFILE_NAME,
            "ck_secret_value_redacted",
            std::collections::HashMap::new(),
            true,
        )
        .expect("store");

    assert_eq!(
        get_composio_api_key(&config).expect("read stored"),
        Some("ck_secret_value_redacted".into())
    );

    // Clearing the profile must remove it again.
    crate::security::credentials::AuthService::from_config(&config)
        .remove_profile(
            COMPOSIO_DIRECT_PROVIDER,
            crate::security::credentials::DEFAULT_AUTH_PROFILE_NAME,
        )
        .expect("remove");
    assert_eq!(
        get_composio_api_key(&config).expect("read post-clear"),
        None
    );
}

#[tokio::test]
async fn direct_list_connections_stops_hitting_composio_after_repeated_invalid_api_key() {
    let _module = module_guard().await;
    let tmp = tempfile::tempdir().unwrap();
    let config = module_test_config(&tmp);
    let hits = Arc::new(AtomicUsize::new(0));
    let app = Router::new()
        .route(
            "/connected_accounts",
            get(|State(hits): State<Arc<AtomicUsize>>| async move {
                hits.fetch_add(1, Ordering::SeqCst);
                (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({ "error": { "message": "Invalid API key" } })),
                )
            }),
        )
        .with_state(hits.clone());
    let base = start_mock_backend(app).await;
    let tool = direct_tool_for_mock_with_key(base, "ck_test_direct_invalid_backoff");
    let _auth_guard = DirectAuthFailureGuard::for_tool(&tool);

    for _ in 0..2 {
        let err = direct_list_connections(&config, &tool)
            .await
            .expect_err("invalid key should reject");
        assert!(
            err.to_string().contains("Invalid API key"),
            "unexpected error: {err:#}"
        );
    }

    let opened = direct_list_connections(&config, &tool)
        .await
        .expect_err("third invalid-key failure should open the backoff gate");
    assert!(
        opened.to_string().contains("re-enter"),
        "backoff error should be actionable, got: {opened:#}"
    );

    let short_circuit = direct_list_connections(&config, &tool)
        .await
        .expect_err("open backoff gate should short-circuit before HTTP");
    assert!(
        short_circuit.to_string().contains("re-enter"),
        "short-circuit error should stay actionable, got: {short_circuit:#}"
    );
    assert_eq!(
        hits.load(Ordering::SeqCst),
        3,
        "after three invalid-key failures, later polls must not hit Composio"
    );
}

#[tokio::test]
async fn direct_list_tools_forwards_tags_and_reshapes_v3_envelope() {
    let _module = module_guard().await;
    let tmp = tempfile::tempdir().unwrap();
    let config = module_test_config(&tmp);
    use axum::extract::RawQuery;
    use std::sync::Mutex;

    let captured: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let sink = captured.clone();
    let app = Router::new().route(
        "/tools",
        get(move |RawQuery(q): RawQuery| {
            let sink = sink.clone();
            async move {
                *sink.lock().unwrap() = q;
                Json(json!({
                    "items": [
                        {
                            "slug": "GITHUB_STAR_A_REPOSITORY",
                            "description": "Star a repository",
                            "input_parameters": { "type": "object" },
                            "toolkit": { "slug": "github" }
                        },
                        // Empty-slug rows must be dropped by the reshaper.
                        { "slug": "", "description": "junk" }
                    ]
                }))
            }
        }),
    );
    let base = start_mock_backend(app).await;
    let tool = direct_tool_for_mock(base);

    let resp = super::direct_list_tools(
        &config,
        &tool,
        &["github".to_string()],
        Some(&["stars".to_string(), "repos".to_string()]),
    )
    .await
    .expect("direct_list_tools should succeed against the mock");

    // Outbound: tags forwarded as repeated params, toolkits CSV.
    let query = captured.lock().unwrap().clone().expect("server saw query");
    assert!(query.contains("tags=stars"), "query was: {query}");
    assert!(query.contains("tags=repos"), "query was: {query}");
    assert!(query.contains("toolkits=github"), "query was: {query}");

    // Inbound: reshaped into the backend envelope, empty-slug row dropped.
    assert_eq!(resp.tools.len(), 1);
    assert_eq!(resp.tools[0].function.name, "GITHUB_STAR_A_REPOSITORY");
    assert_eq!(resp.tools[0].kind, "function");
}

#[tokio::test]
async fn pricing_for_config_short_circuits_in_direct_mode() {
    // Build a client pointed at an unreachable backend — if the
    // short-circuit fires, we never actually attempt the network call
    // and the empty default struct comes back immediately.
    let client =
        crate::integrations::IntegrationClient::new("http://127.0.0.1:0".into(), "test".into());
    let mut config = crate::config::Config::default();
    config.composio.mode = "direct".into();

    let pricing = crate::integrations::pricing_for_config(&client, &config).await;
    // The default struct has every per-integration entry as `None`.
    assert!(pricing.integrations.apify.is_none());
    assert!(pricing.integrations.twilio.is_none());
    assert!(pricing.integrations.google_places.is_none());
    assert!(pricing.integrations.parallel.is_none());
    assert!(pricing.integrations.tinyfish.is_none());
}
