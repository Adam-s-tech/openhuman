use super::*;
use axum::{body::Bytes, routing::post, Router};
use std::sync::{Arc, Mutex};

#[test]
fn backend_401_is_tagged_as_session_expiry() {
    let tagged = classify_transcribe_error(
        "transcription request failed (401 Unauthorized): {\"error\":\"Invalid token\"}"
            .to_string(),
        false,
    );
    assert!(tagged.starts_with("SESSION_EXPIRED: "));
    assert!(crate::core::observability::is_session_expired_message(
        &tagged
    ));
}

#[test]
fn other_failures_pass_through_unchanged() {
    let error = "transcription request failed (500 Internal Server Error): boom".to_string();
    assert_eq!(classify_transcribe_error(error.clone(), false), error);
}

#[test]
fn backend_401_with_api_key_does_not_expire_session() {
    let tagged = classify_transcribe_error(
        "transcription request failed (401 Unauthorized)".to_string(),
        true,
    );
    assert!(tagged.starts_with("API_KEY_REJECTED: "));
    assert!(!crate::core::observability::is_session_expired_message(
        &tagged
    ));
}

#[tokio::test]
async fn offline_local_session_refuses_without_a_request() {
    let tmp = tempfile::TempDir::new().unwrap();
    let config = Config {
        workspace_dir: tmp.path().join("workspace"),
        action_dir: tmp.path().join("workspace"),
        config_path: tmp.path().join("config.toml"),
        ..Config::default()
    };
    crate::security::credentials::AuthService::from_config(&config)
        .store_provider_token(
            crate::security::credentials::APP_SESSION_PROVIDER,
            crate::security::credentials::DEFAULT_AUTH_PROFILE_NAME,
            "desktop.test.local",
            std::collections::HashMap::new(),
            true,
        )
        .unwrap();
    let err = transcribe_cloud(&config, "AAAA", &CloudTranscribeOptions::default())
        .await
        .unwrap_err();
    assert!(
        crate::core::observability::is_backend_unavailable_message(&err),
        "{err}"
    );
}

#[tokio::test]
async fn api_key_refuses_remote_plaintext_endpoint_before_request() {
    let tmp = tempfile::TempDir::new().unwrap();
    let config = Config {
        workspace_dir: tmp.path().join("workspace"),
        action_dir: tmp.path().join("workspace"),
        config_path: tmp.path().join("config.toml"),
        api_url: Some("http://backend.example.test".to_string()),
        ..Config::default()
    };
    crate::security::credentials::api_key::store_api_key(&config, "test-api-key").unwrap();

    let err = transcribe_cloud(&config, "AAAA", &CloudTranscribeOptions::default())
        .await
        .unwrap_err();
    assert!(err.contains("refusing to send"), "{err}");
}

#[tokio::test]
async fn api_key_transcribes_via_safe_backend_with_bearer_and_multipart_audio() {
    let request = Arc::new(Mutex::new(None));
    let captured = Arc::clone(&request);
    let app = Router::new().route(
        "/openai/v1/audio/transcriptions",
        post(move |headers: axum::http::HeaderMap, body: Bytes| {
            let captured = Arc::clone(&captured);
            async move {
                *captured.lock().unwrap() = Some((
                    headers
                        .get(axum::http::header::AUTHORIZATION)
                        .and_then(|value| value.to_str().ok())
                        .map(str::to_owned),
                    headers
                        .get(axum::http::header::CONTENT_TYPE)
                        .and_then(|value| value.to_str().ok())
                        .map(str::to_owned),
                    body.to_vec(),
                ));
                (axum::http::StatusCode::OK, r#"{"text":"recognized words"}"#)
            }
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let endpoint = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

    let tmp = tempfile::TempDir::new().unwrap();
    let config = Config {
        workspace_dir: tmp.path().join("workspace"),
        action_dir: tmp.path().join("workspace"),
        config_path: tmp.path().join("config.toml"),
        api_url: Some(endpoint),
        ..Config::default()
    };
    crate::security::credentials::api_key::store_api_key(&config, "test-api-key").unwrap();

    let outcome = transcribe_cloud(&config, "AQID", &CloudTranscribeOptions::default())
        .await
        .unwrap()
        .value;
    assert_eq!(outcome.text, "recognized words");
    let (authorization, content_type, body) = request.lock().unwrap().take().unwrap();
    assert_eq!(authorization.as_deref(), Some("Bearer test-api-key"));
    assert!(content_type
        .as_deref()
        .unwrap()
        .starts_with("multipart/form-data; boundary="));
    assert!(body.windows(3).any(|bytes| bytes == [1, 2, 3]));
    let body = String::from_utf8_lossy(&body);
    assert!(body.contains("name=\"file\""));
    assert!(body.contains("name=\"model\""));
}
