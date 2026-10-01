use super::*;

#[derive(Serialize, Deserialize)]
struct Holder {
    #[serde(with = "history_wire")]
    rows: Vec<TranscriptMessage>,
    #[serde(default, with = "history_wire::option")]
    maybe: Option<Vec<TranscriptMessage>>,
}

#[test]
fn history_wire_writes_only_role_and_content() {
    let mut row = TranscriptMessage::tool("{\"tool_call_id\":\"c1\",\"content\":\"ok\"}");
    row.id = Some("c1".into());
    row.cache_breakpoints = vec![3, 7];
    row.extra_metadata = Some(serde_json::json!({"reasoning_content": "why"}));
    let holder = Holder {
        rows: vec![row],
        maybe: None,
    };
    assert_eq!(
        serde_json::to_value(&holder).unwrap(),
        serde_json::json!({
            "rows": [{"role": "tool", "content": "{\"tool_call_id\":\"c1\",\"content\":\"ok\"}"}],
            "maybe": null,
        })
    );
}

#[test]
fn history_wire_reads_bare_and_full_rows() {
    let holder: Holder = serde_json::from_value(serde_json::json!({
        "rows": [
            {"role": "user", "content": "hi"},
            {"id": "m1", "role": "system", "content": "s", "cache_breakpoints": [3, 7]},
        ],
        "maybe": [{"role": "assistant", "content": "a"}],
    }))
    .unwrap();
    assert_eq!(holder.rows[0], TranscriptMessage::user("hi"));
    assert_eq!(holder.rows[1].id.as_deref(), Some("m1"));
    assert_eq!(holder.rows[1].cache_breakpoints, vec![3, 7]);
    assert_eq!(holder.maybe, Some(vec![TranscriptMessage::assistant("a")]));
}

#[test]
fn history_wire_option_defaults_to_none_when_absent() {
    let holder: Holder = serde_json::from_value(serde_json::json!({"rows": []})).unwrap();
    assert!(holder.rows.is_empty() && holder.maybe.is_none());
}
