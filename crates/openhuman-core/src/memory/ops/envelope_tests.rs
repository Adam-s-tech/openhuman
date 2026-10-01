use super::*;

#[test]
fn memory_counts_collects_named_entries() {
    let counts = memory_counts([("documents", 2), ("entities", 5)]);
    assert_eq!(counts.get("documents"), Some(&2));
    assert_eq!(counts.get("entities"), Some(&5));
}

#[test]
fn envelope_wraps_data_with_meta() {
    let outcome = envelope(serde_json::json!({"ok": true}), None, None);
    let value = outcome.into_cli_compatible_json().unwrap();
    assert_eq!(value["data"]["ok"], true);
    assert!(value["meta"]["request_id"].as_str().is_some());
    assert!(value["error"].is_null());
}

#[test]
fn error_envelope_wraps_code_and_message() {
    let outcome: Outcome<ApiEnvelope<serde_json::Value>> =
        error_envelope("bad_request", "boom".to_string());
    let value = outcome.into_cli_compatible_json().unwrap();
    assert_eq!(value["error"]["code"], "bad_request");
    assert_eq!(value["error"]["message"], "boom");
    assert!(value["data"].is_null());
}

#[test]
fn envelope_propagates_counts_and_pagination_and_leaves_the_rest_unset() {
    let counts = memory_counts([("num_messages", 7)]);
    let pagination = PaginationMeta {
        limit: 10,
        offset: 0,
        count: 7,
    };
    let out = envelope(
        serde_json::json!({"v": 42}),
        Some(counts.clone()),
        Some(pagination),
    );
    let env = &out.value;
    assert_eq!(env.data.as_ref().unwrap()["v"], 42);
    assert!(env.error.is_none());
    assert_eq!(env.meta.counts.as_ref().unwrap(), &counts);
    let pag = env.meta.pagination.as_ref().unwrap();
    assert_eq!((pag.limit, pag.offset, pag.count), (10, 0, 7));
    assert!(env.meta.latency_seconds.is_none() && env.meta.cached.is_none());
    assert!(out.logs.is_empty());
}

#[test]
fn memory_request_id_is_a_fresh_v4_uuid() {
    let (a, b) = (memory_request_id(), memory_request_id());
    assert_eq!(a.len(), 36);
    assert_ne!(a, b);
}
