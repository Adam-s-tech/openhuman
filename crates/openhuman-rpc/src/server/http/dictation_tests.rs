use std::sync::Once;

use axum::body::Body;
use axum::http::{header, HeaderMap, HeaderValue, Request, StatusCode};
use tower::ServiceExt;

use super::{authorize_dictation_request, DictationQuery};

fn test_token() -> String {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        openhuman_core::core::auth::init_rpc_token_with_value("dictation-http-tests-token")
            .expect("initialize test bearer");
    });
    openhuman_core::core::auth::get_rpc_token()
        .expect("test bearer initialized")
        .to_string()
}

fn query(token: Option<&str>) -> DictationQuery {
    DictationQuery {
        token: token.map(str::to_string),
    }
}

#[test]
fn dictation_rejects_disallowed_origin_before_authentication() {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::ORIGIN,
        HeaderValue::from_static("https://attacker.example"),
    );

    let response = authorize_dictation_request(&headers, &query(Some(&test_token())))
        .expect_err("cross-origin request rejected");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[test]
fn dictation_rejects_missing_and_invalid_credentials() {
    for query in [query(None), query(Some("invalid"))] {
        let response = authorize_dictation_request(&HeaderMap::new(), &query)
            .expect_err("missing or invalid token rejected");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}

#[test]
fn dictation_accepts_bearer_header_and_browser_query_token() {
    let token = test_token();
    let mut headers = HeaderMap::new();
    headers.insert(
        header::AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {token}")).unwrap(),
    );
    assert!(authorize_dictation_request(&headers, &query(None)).is_ok());

    assert!(authorize_dictation_request(&HeaderMap::new(), &query(Some(&token))).is_ok());
}

#[tokio::test]
async fn dictation_handler_rejects_disallowed_origin_before_upgrade() {
    let request = Request::builder()
        .uri("/ws/dictation")
        .header(header::ORIGIN, "https://attacker.example")
        .header(header::AUTHORIZATION, format!("Bearer {}", test_token()))
        .header(header::CONNECTION, "upgrade")
        .header(header::UPGRADE, "websocket")
        .header("sec-websocket-version", "13")
        .header("sec-websocket-key", "dGhlIHNhbXBsZSBub25jZQ==")
        .body(Body::empty())
        .unwrap();

    let response = crate::server::http::build_core_http_router(false)
        .oneshot(request)
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
