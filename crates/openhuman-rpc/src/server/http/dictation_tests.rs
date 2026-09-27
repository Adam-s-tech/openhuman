use axum::body::{to_bytes, Body};
use axum::http::{header, Request, StatusCode};
use axum::routing::get;
use axum::Router;
use tower::ServiceExt;

use super::dictation_ws_handler;

fn upgrade_request(uri: &str, origin: Option<&str>) -> Request<Body> {
    let mut builder = Request::builder()
        .uri(uri)
        .header(header::CONNECTION, "Upgrade")
        .header(header::UPGRADE, "websocket")
        .header("sec-websocket-version", "13")
        .header("sec-websocket-key", "dGhlIHNhbXBsZSBub25jZQ==");
    if let Some(origin) = origin {
        builder = builder.header(header::ORIGIN, origin);
    }
    builder.body(Body::empty()).unwrap()
}

async fn dispatch(request: Request<Body>) -> axum::response::Response {
    Router::new()
        .route("/ws/dictation", get(dictation_ws_handler))
        .oneshot(request)
        .await
        .unwrap()
}

#[tokio::test]
async fn dictation_rejects_foreign_browser_origins_before_authentication() {
    let response = dispatch(upgrade_request(
        "/ws/dictation?token=not-a-bearer",
        Some("https://attacker.example"),
    ))
    .await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert!(String::from_utf8_lossy(&body).contains("Origin not allowed"));
}

#[tokio::test]
async fn dictation_rejects_missing_and_invalid_credentials() {
    for uri in ["/ws/dictation", "/ws/dictation?token=not-a-bearer"] {
        let response = dispatch(upgrade_request(uri, None)).await;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert!(String::from_utf8_lossy(&body).contains("Missing or invalid token"));
    }
}
