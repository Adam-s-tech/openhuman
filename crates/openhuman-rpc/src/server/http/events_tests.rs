use axum::body::to_bytes;
use axum::extract::Query;
use axum::http::{header, HeaderMap, StatusCode};

use super::{domain_events_handler, events_handler, webhook_events_handler, EventsQuery};

fn query(client_id: &str, token: Option<&str>) -> Query<EventsQuery> {
    Query(EventsQuery {
        client_id: client_id.to_string(),
        token: token.map(str::to_string),
    })
}

#[tokio::test]
async fn events_require_a_credential_and_reject_unknown_bind_tokens() {
    let missing = events_handler(HeaderMap::new(), query("client", None)).await;
    assert_eq!(missing.status(), StatusCode::UNAUTHORIZED);
    let body = to_bytes(missing.into_body(), usize::MAX).await.unwrap();
    assert!(String::from_utf8_lossy(&body).contains("Missing credentials"));

    let invalid = events_handler(HeaderMap::new(), query("client", Some("unknown"))).await;
    assert_eq!(invalid.status(), StatusCode::UNAUTHORIZED);
    let body = to_bytes(invalid.into_body(), usize::MAX).await.unwrap();
    assert!(String::from_utf8_lossy(&body).contains("unknown, expired"));
}

#[tokio::test]
async fn events_bind_token_is_client_bound_and_single_use() {
    let token = openhuman_core::core::event_bind_tokens::issue("right", None)
        .expect("bind token")
        .token;

    let wrong = events_handler(HeaderMap::new(), query("wrong", Some(&token))).await;
    assert_eq!(wrong.status(), StatusCode::UNAUTHORIZED);

    let accepted = events_handler(HeaderMap::new(), query("right", Some(&token))).await;
    assert_eq!(accepted.status(), StatusCode::OK);
    assert_eq!(
        accepted.headers()[header::CONTENT_TYPE],
        "text/event-stream"
    );
    drop(accepted);

    let replay = events_handler(HeaderMap::new(), query("right", Some(&token))).await;
    assert_eq!(replay.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn events_stream_forwards_only_the_bound_client() {
    use openhuman_core::web_chat::{publish_web_channel_event, WebChannelEvent};
    use tokio_stream::StreamExt;

    let token = openhuman_core::core::event_bind_tokens::issue("stream-client", None)
        .expect("bind token")
        .token;
    let response = events_handler(HeaderMap::new(), query("stream-client", Some(&token))).await;
    assert_eq!(response.status(), StatusCode::OK);
    let mut body = response.into_body().into_data_stream();

    publish_web_channel_event(WebChannelEvent {
        event: "wrong_client_probe".into(),
        client_id: "another-client".into(),
        ..Default::default()
    });
    publish_web_channel_event(WebChannelEvent {
        event: "right_client_probe".into(),
        client_id: "stream-client".into(),
        ..Default::default()
    });

    let chunk = tokio::time::timeout(std::time::Duration::from_secs(1), body.next())
        .await
        .expect("matching event arrived")
        .expect("SSE body chunk")
        .expect("SSE bytes");
    let event = String::from_utf8_lossy(&chunk);
    assert!(event.contains("event: right_client_probe"));
    assert!(!event.contains("wrong_client_probe"));
}

#[tokio::test]
async fn domain_events_require_a_bearer() {
    let response = domain_events_handler(HeaderMap::new()).await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert!(String::from_utf8_lossy(&body).contains("Bearer token required"));
}

#[tokio::test]
async fn webhook_debug_stream_starts_with_a_documented_event() {
    let response = webhook_events_handler().await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers()[header::CONTENT_TYPE],
        "text/event-stream"
    );
    let mut body = response.into_body().into_data_stream();
    use tokio_stream::StreamExt;
    let chunk = body.next().await.expect("first event").expect("SSE bytes");
    let event = String::from_utf8_lossy(&chunk);
    assert!(event.contains("event: webhooks_debug"));
    assert!(event.contains("runtime_removed"));
}
