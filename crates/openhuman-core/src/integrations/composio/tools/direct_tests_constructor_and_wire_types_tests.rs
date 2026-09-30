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

    let tool = ComposioTool::new_with_v3_base(
        "ck_secret_value",
        None,
        test_security(),
        format!("{source}/tools"),
    );
    assert!(tool.list_tool_schemas_v3(&[], None).await.is_err());
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

#[test]
fn build_list_tool_schemas_v3_query_always_includes_limit() {
    let params = ComposioTool::build_list_tool_schemas_v3_query(&[], None);
    assert_eq!(
        params,
        vec![
            ("limit", "200".to_string()),
            ("toolkit_versions", "latest".to_string()),
        ]
    );
}

#[test]
fn build_list_tool_schemas_v3_query_joins_toolkits_as_csv() {
    let params = ComposioTool::build_list_tool_schemas_v3_query(&["github", "gmail"], None);
    assert_eq!(
        params,
        vec![
            ("limit", "200".to_string()),
            ("toolkit_versions", "latest".to_string()),
            ("toolkits", "github,gmail".to_string()),
        ]
    );
}

#[test]
fn build_list_tool_schemas_v3_query_emits_repeated_tags_params() {
    // Composio v3 `/tools` takes tags as repeated `tags=` params
    // (tags=stars&tags=repos), NOT comma-joined like the backend proxy.
    // A Vec of duplicate ("tags", _) keys is exactly what reqwest's
    // `.query(&params)` serializes into repeated query params.
    let params =
        ComposioTool::build_list_tool_schemas_v3_query(&["github"], Some(&["stars", "repos"]));
    assert_eq!(
        params,
        vec![
            ("limit", "200".to_string()),
            ("toolkit_versions", "latest".to_string()),
            ("toolkits", "github".to_string()),
            ("tags", "stars".to_string()),
            ("tags", "repos".to_string()),
        ]
    );
}

#[test]
fn build_list_tool_schemas_v3_query_tags_without_toolkit_filter() {
    let params = ComposioTool::build_list_tool_schemas_v3_query(&[], Some(&["readOnlyHint"]));
    assert_eq!(
        params,
        vec![
            ("limit", "200".to_string()),
            ("toolkit_versions", "latest".to_string()),
            ("tags", "readOnlyHint".to_string()),
        ]
    );
}

#[test]
fn build_list_tool_schemas_v3_query_trims_and_drops_blank_entries() {
    let params = ComposioTool::build_list_tool_schemas_v3_query(
        &["  github  ", "   "],
        Some(&["  stars  ", "", "   "]),
    );
    assert_eq!(
        params,
        vec![
            ("limit", "200".to_string()),
            ("toolkit_versions", "latest".to_string()),
            ("toolkits", "github".to_string()),
            ("tags", "stars".to_string()),
        ]
    );
}

#[test]
fn build_list_tool_schemas_v3_query_empty_tags_slice_is_no_filter() {
    // `Some(&[])` and an all-blank slice must both behave like "no tags".
    let empty = ComposioTool::build_list_tool_schemas_v3_query(&["gmail"], Some(&[]));
    let blank = ComposioTool::build_list_tool_schemas_v3_query(&["gmail"], Some(&["  "]));
    let expected = vec![
        ("limit", "200".to_string()),
        ("toolkit_versions", "latest".to_string()),
        ("toolkits", "gmail".to_string()),
    ];
    assert_eq!(empty, expected);
    assert_eq!(blank, expected);
}

#[test]
fn build_list_tool_schemas_v3_query_pins_toolkit_versions_latest() {
    // #3932: without toolkit_versions, Composio v3 defaults to the pinned
    // 00000000_00 snapshot, so any toolkit published after it (Outlook and
    // every other post-launch toolkit) lists zero tools. `latest` keeps them
    // visible.
    let params = ComposioTool::build_list_tool_schemas_v3_query(&["outlook"], None);
    assert!(
        params.contains(&("toolkit_versions", "latest".to_string())),
        "query must pin toolkit_versions=latest; got {params:?}"
    );
}

// ── list_tool_schemas_v3 over HTTP (direct-mode tags reach the wire) ───────

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

    let tool = ComposioTool::new_with_v3_base("ck_test_direct", None, test_security(), base);
    let items = tool
        .list_tool_schemas_v3(&["github"], Some(&["stars", "repos"]))
        .await
        .expect("direct v3 /tools should succeed against the mock");

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
    assert_eq!(items[0].slug, "GITHUB_STAR_A_REPOSITORY");
    assert_eq!(items[0].toolkit_slug.as_deref(), Some("github"));
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
