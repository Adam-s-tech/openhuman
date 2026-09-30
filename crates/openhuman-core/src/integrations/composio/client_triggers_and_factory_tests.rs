use super::*;

#[tokio::test]
async fn list_available_triggers_forwards_query_params() {
    let app = Router::new().route(
        "/agent-integrations/composio/triggers/available",
        get(|Query(q): Query<HashMap<String, String>>| async move {
            assert_eq!(q.get("toolkit").map(String::as_str), Some("github"));
            assert_eq!(q.get("connectionId").map(String::as_str), Some("c1"));
            Json(json!({
                "success": true,
                "data": {"triggers": [{"slug": "GITHUB_PUSH_EVENT", "scope": "github_repo"}]}
            }))
        }),
    );
    let base = start_mock_backend(app).await;
    let client = build_client_for(base);
    let resp = client
        .list_available_triggers("github", Some("c1"))
        .await
        .unwrap();
    assert_eq!(resp.triggers.len(), 1);
    assert_eq!(resp.triggers[0].scope, "github_repo");
}

#[tokio::test]
async fn list_active_triggers_filters_by_toolkit() {
    let app = Router::new().route(
        "/agent-integrations/composio/triggers",
        get(|Query(q): Query<HashMap<String, String>>| async move {
            assert_eq!(q.get("toolkit").map(String::as_str), Some("gmail"));
            Json(json!({
                "success": true,
                "data": {"triggers": []}
            }))
        }),
    );
    let base = start_mock_backend(app).await;
    let client = build_client_for(base);
    let resp = client.list_active_triggers(Some("gmail")).await.unwrap();
    assert!(resp.triggers.is_empty());
}

#[tokio::test]
async fn enable_trigger_rejects_empty_inputs() {
    let inner = Arc::new(crate::integrations::IntegrationClient::new(
        "http://127.0.0.1:0".into(),
        "test".into(),
    ));
    let client = ComposioClient::new(inner);

    let err = client.enable_trigger("", "X", None).await.unwrap_err();
    assert!(err.to_string().contains("connectionId must not be empty"));

    let err = client.enable_trigger("c1", "  ", None).await.unwrap_err();
    assert!(err.to_string().contains("slug must not be empty"));
}

#[tokio::test]
async fn enable_trigger_posts_body_and_parses_response() {
    let app = Router::new().route(
        "/agent-integrations/composio/triggers",
        post(|Json(body): Json<Value>| async move {
            assert_eq!(body["connectionId"], "c1");
            assert_eq!(body["slug"], "GMAIL_NEW_GMAIL_MESSAGE");
            assert_eq!(body["triggerConfig"]["labelIds"], "INBOX");
            Json(json!({
                "success": true,
                "data": {
                    "triggerId": "ti_1",
                    "slug": "GMAIL_NEW_GMAIL_MESSAGE",
                    "connectionId": "c1"
                }
            }))
        }),
    );
    let base = start_mock_backend(app).await;
    let client = build_client_for(base);
    let resp = client
        .enable_trigger(
            "c1",
            "GMAIL_NEW_GMAIL_MESSAGE",
            Some(json!({"labelIds": "INBOX"})),
        )
        .await
        .unwrap();
    assert_eq!(resp.trigger_id, "ti_1");
}

#[tokio::test]
async fn disable_trigger_rejects_empty_id() {
    let inner = Arc::new(crate::integrations::IntegrationClient::new(
        "http://127.0.0.1:0".into(),
        "test".into(),
    ));
    let client = ComposioClient::new(inner);
    let err = client.disable_trigger("").await.unwrap_err();
    assert!(err.to_string().contains("triggerId must not be empty"));
}

#[tokio::test]
async fn disable_trigger_calls_delete_path() {
    let app = Router::new().route(
        "/agent-integrations/composio/triggers/{id}",
        axum::routing::delete(|Path(id): Path<String>| async move {
            assert_eq!(id, "ti_1");
            Json(json!({"success": true, "data": {"deleted": true}}))
        }),
    );
    let base = start_mock_backend(app).await;
    let client = build_client_for(base);
    let resp = client.disable_trigger("ti_1").await.unwrap();
    assert!(resp.deleted);
}

#[tokio::test]
async fn disable_trigger_surfaces_non_2xx_status() {
    let app = Router::new().route(
        "/agent-integrations/composio/triggers/{id}",
        axum::routing::delete(|Path(_id): Path<String>| async move {
            (
                StatusCode::NOT_FOUND,
                Json(json!({"success": false, "error": "no"})),
            )
        }),
    );
    let base = start_mock_backend(app).await;
    let client = build_client_for(base);
    let err = client.disable_trigger("ti_x").await.unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("404"), "expected status 404, got: {msg}");
    // Phase A (#1296): raw_delete must propagate the envelope's `error`
    // field so callers can tell *why* the backend rejected the call.
    assert!(
        msg.contains("no"),
        "expected envelope error detail in message, got: {msg}"
    );
}

#[tokio::test]
async fn delete_connection_surfaces_envelope_error_detail() {
    // Direct cover of the `raw_delete` envelope-error path used by
    // `delete_connection` — proves the backend message ("Connection
    // not found") makes it into the propagated bail message rather
    // than being discarded with the body. Mirror of the `post`/`get`
    // envelope tests in `integrations/client_tests.rs`.
    let app = Router::new().route(
        "/agent-integrations/composio/connections/{id}",
        axum::routing::delete(|Path(_id): Path<String>| async move {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({"success": false, "error": "Connection not found"})),
            )
        }),
    );
    let base = start_mock_backend(app).await;
    let client = build_client_for(base);
    let err = client.delete_connection("missing-id").await.unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("Connection not found"),
        "expected backend error detail in message, got: {msg}"
    );
    assert!(msg.contains("400"), "expected status 400, got: {msg}");
}

// ── execute_tool resilience tests (Batch 1 — post-OAuth readiness) ─────

#[test]
fn create_composio_client_backend_variant_when_mode_default() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let config = config_with_session_token(&tmp);
    let kind = create_composio_client(&config).expect("backend mode should build");
    assert_eq!(kind.mode(), "backend");
    assert!(matches!(kind, ComposioClientKind::Backend(_)));
}

#[test]
fn create_composio_client_backend_empty_mode_falls_back_to_backend() {
    // A literal empty string in TOML should be treated as the default
    // (`"backend"`) rather than an unknown mode error.
    let tmp = tempfile::tempdir().expect("tempdir");
    let mut config = config_with_session_token(&tmp);
    config.composio.mode = String::new();
    let kind = create_composio_client(&config).expect("empty mode should fall back to backend");
    assert_eq!(kind.mode(), "backend");
}

#[test]
fn create_composio_client_backend_errors_without_session() {
    // Backend mode requires the app-session JWT — without it the
    // factory must return an explicit error (not silently downgrade).
    let tmp = tempfile::tempdir().expect("tempdir");
    let mut config = crate::config::Config::default();
    config.config_path = tmp.path().join("config.toml");
    let err = create_composio_client(&config)
        .err()
        .expect("must error without auth token");
    assert!(
        err.to_string().contains("no backend session token"),
        "unexpected error: {err}"
    );
}

#[test]
fn create_composio_client_direct_variant_with_stored_key() {
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
    let kind = create_composio_client(&config).expect("direct mode with stored key should build");
    assert_eq!(kind.mode(), "direct");
    assert!(matches!(kind, ComposioClientKind::Direct(_)));
}

#[test]
fn create_composio_client_direct_falls_back_to_config_api_key() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let mut config = crate::config::Config::default();
    config.config_path = tmp.path().join("config.toml");
    config.composio.mode = "direct".into();
    // No keychain entry — fall back to the inline config field.
    config.composio.api_key = Some("ck_inline_redacted".into());
    let kind = create_composio_client(&config)
        .expect("direct mode should accept inline config.api_key when keychain is empty");
    assert_eq!(kind.mode(), "direct");
}

#[test]
fn create_composio_client_direct_errors_without_key() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let mut config = crate::config::Config::default();
    config.config_path = tmp.path().join("config.toml");
    config.composio.mode = "direct".into();
    let err = create_composio_client(&config)
        .err()
        .expect("direct without key must error");
    let msg = err.to_string();
    assert!(
        msg.contains("no api key is configured"),
        "unexpected error: {msg}"
    );
}

#[test]
fn create_composio_client_unknown_mode_errors() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let mut config = crate::config::Config::default();
    config.config_path = tmp.path().join("config.toml");
    config.composio.mode = "voyage".into();
    let err = create_composio_client(&config)
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
        let err = direct_list_connections(&tool)
            .await
            .expect_err("invalid key should reject");
        assert!(
            err.to_string().contains("Invalid API key"),
            "unexpected error: {err:#}"
        );
    }

    let opened = direct_list_connections(&tool)
        .await
        .expect_err("third invalid-key failure should open the backoff gate");
    assert!(
        opened.to_string().contains("re-enter"),
        "backoff error should be actionable, got: {opened:#}"
    );

    let short_circuit = direct_list_connections(&tool)
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

    let resp = super::super::direct_list_tools(
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
