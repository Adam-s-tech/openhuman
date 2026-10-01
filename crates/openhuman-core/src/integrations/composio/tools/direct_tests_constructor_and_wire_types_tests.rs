use super::*;

use std::sync::Arc;
// ── Constructor ───────────────────────────────────────────

#[tokio::test]
async fn direct_client_does_not_forward_credentials_across_redirects() {
    let redirected_request_seen = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let observed = redirected_request_seen.clone();
    let destination = start_mock_backend(axum::Router::new().route(
        "/tools",
        axum::routing::get(move || {
            let observed = observed.clone();
            async move {
                observed.store(true, std::sync::atomic::Ordering::SeqCst);
                axum::Json(json!({"items": []}))
            }
        }),
    ))
    .await;
    let redirect = format!("{destination}/tools");
    let source = start_mock_backend(axum::Router::new().route(
        "/tools",
        axum::routing::get(move || {
            let redirect = redirect.clone();
            async move { axum::response::Redirect::temporary(&redirect) }
        }),
    ))
    .await;

    let tool = Arc::new(DirectComposioClient::new_with_v3_base(
        "ck_secret_value",
        source,
    ));
    assert!(tool.list_tools(&[], &[]).await.is_err());
    assert!(!redirected_request_seen.load(std::sync::atomic::Ordering::SeqCst));
}

// ── Execute validation ────────────────────────────────────

// ── API response parsing ──────────────────────────────────

#[test]
fn extract_api_error_message_from_common_shapes() {
    let nested = r#"{"error":{"message":"tool not found"}}"#;
    let flat = r#"{"message":"invalid api key"}"#;

    assert_eq!(
        extract_api_error_message(nested).as_deref(),
        Some("tool not found")
    );
    assert_eq!(
        extract_api_error_message(flat).as_deref(),
        Some("invalid api key")
    );
    assert_eq!(extract_api_error_message("not-json"), None);
}

#[test]
fn composio_api_base_url_is_v3() {
    assert_eq!(COMPOSIO_API_BASE_V3, "https://backend.composio.dev/api/v3");
}

// ── list_tool_schemas_v3 query builder (direct-mode tags) ──────────────────

// ── direct list_tools over HTTP (direct-mode tags reach the wire) ───────

#[tokio::test]
async fn list_tool_schemas_v3_sends_repeated_tags_to_v3_tools_endpoint() {
    use axum::{extract::RawQuery, routing::get, Json, Router};
    use std::sync::Mutex;

    // Capture the raw query string the server sees. `RawQuery` (not
    // `Query<HashMap>`) is required because a HashMap would collapse the
    // repeated `tags=` params we specifically need to assert on.
    let captured: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let sink = captured.clone();
    let app = Router::new().route(
        "/tools",
        get(move |RawQuery(q): RawQuery| {
            let sink = sink.clone();
            async move {
                *sink.lock().unwrap() = q;
                Json(json!({
                    "items": [{
                        "slug": "GITHUB_STAR_A_REPOSITORY",
                        "description": "Star a repository",
                        "input_parameters": { "type": "object" },
                        "toolkit": { "slug": "github" }
                    }]
                }))
            }
        }),
    );
    let base = start_mock_backend(app).await;

    let tool = Arc::new(DirectComposioClient::new_with_v3_base(
        "ck_test_direct",
        base,
    ));
    let items = tool
        .list_tools(
            &["github".to_string()],
            &["stars".to_string(), "repos".to_string()],
        )
        .await
        .expect("direct v3 /tools should succeed against the mock")
        .tools;

    let query = captured
        .lock()
        .unwrap()
        .clone()
        .expect("mock server should have observed a query string");

    // tags must be REPEATED params (tags=stars&tags=repos) — the Composio v3
    // contract — NOT the comma-joined form the backend proxy uses.
    assert!(query.contains("tags=stars"), "query was: {query}");
    assert!(query.contains("tags=repos"), "query was: {query}");
    assert!(
        !query.contains("stars%2Crepos") && !query.contains("stars,repos"),
        "tags must not be comma-joined; query was: {query}"
    );
    assert!(query.contains("toolkits=github"), "query was: {query}");
    assert!(query.contains("limit=200"), "query was: {query}");
    // #3932: post-launch toolkits are invisible without toolkit_versions=latest.
    assert!(
        query.contains("toolkit_versions=latest"),
        "query was: {query}"
    );

    // And the v3 envelope reshapes back into schema items.
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].function.name, "GITHUB_STAR_A_REPOSITORY");
    assert_eq!(
        items[0].function.description.as_deref(),
        Some("Star a repository")
    );
}

// ── execute_action over HTTP (correct v3 path/slug/body reach the wire) ────

// ── ensure_https ──────────────────────────────────────────────────────────

#[test]
fn ensure_https_accepts_https_url() {
    assert!(ensure_https("https://backend.composio.dev/api/v3/tools").is_ok());
}

#[test]
fn ensure_https_rejects_http_url() {
    let err = ensure_https("http://backend.composio.dev/api/v3/tools").unwrap_err();
    assert!(err.to_string().contains("non-HTTPS"));
}

#[test]
fn ensure_https_rejects_ftp_url() {
    assert!(ensure_https("ftp://example.com").is_err());
}

// ── sanitize_error_message ────────────────────────────────────────────────

#[test]
fn sanitize_error_message_replaces_sensitive_fields() {
    let msg = "Invalid connected_account_id value for entity_id: user-123";
    let sanitized = sanitize_error_message(msg);
    assert!(!sanitized.contains("connected_account_id"));
    assert!(!sanitized.contains("entity_id"));
    assert!(sanitized.contains("[redacted]"));
}

#[test]
fn sanitize_error_message_replaces_newlines_with_spaces() {
    let msg = "line1\nline2\nline3";
    let sanitized = sanitize_error_message(msg);
    assert!(!sanitized.contains('\n'));
    assert!(sanitized.contains("line1"));
    assert!(sanitized.contains("line2"));
}

// ── failure messages stay byte-identical to the pre-module client ─────────

#[tokio::test]
async fn http_failures_keep_their_user_facing_messages() {
    use axum::{http::StatusCode, routing::get, Json, Router};
    let app = Router::new()
        .route(
            "/connected_accounts",
            get(|| async {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({"error": {"message": "Invalid API key"}})),
                )
            }),
        )
        .route(
            "/tools",
            get(|| async { (StatusCode::INTERNAL_SERVER_ERROR, "") }),
        );
    let tool = Arc::new(DirectComposioClient::new_with_v3_base(
        "ck_test_direct",
        start_mock_backend(app).await,
    ));

    let err = tool.list_connections().await.unwrap_err();
    assert_eq!(
        format!("{err:#}"),
        "Composio v3 connected_accounts failed: HTTP 401: Invalid API key"
    );
    let err = tool.list_tools(&[], &[]).await.unwrap_err();
    assert_eq!(
        format!("{err:#}"),
        "Composio v3 list_tool_schemas: HTTP 500"
    );
}

#[tokio::test]
async fn connections_come_back_without_route_lifted_identity() {
    use axum::{routing::get, Json, Router};
    let app = Router::new().route(
        "/connected_accounts",
        get(|| async {
            Json(json!({"items": [
                {"id": " ca_1 ", "toolkit": "gmail", "status": "ACTIVE", "email": "a@b.c"},
                {"id": "  ", "toolkit": "slack", "status": "ACTIVE"}
            ]}))
        }),
    );
    let tool = Arc::new(DirectComposioClient::new_with_v3_base(
        "ck_test_direct",
        start_mock_backend(app).await,
    ));
    let connections = tool.list_connections().await.unwrap().connections;
    assert_eq!(connections.len(), 1, "blank id dropped");
    assert_eq!(connections[0].id, "ca_1");
    // Identity is the host's to enrich from cached profiles.
    assert!(connections[0].account_email.is_none());
}
