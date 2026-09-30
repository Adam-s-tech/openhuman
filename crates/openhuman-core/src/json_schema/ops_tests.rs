use super::*;
use serde_json::{json, Value};

#[test]
fn unsupported_arg_names_empty_when_every_name_is_a_real_property() {
    let schema = json!({
        "type": "object",
        "properties": { "channel": {"type": "string"}, "markdown_text": {"type": "string"} }
    });
    let args = json!({ "channel": "#general", "markdown_text": "hi" });
    assert_eq!(unsupported_arg_names(Some(&schema), &args), Some(vec![]));
}

#[test]
fn unsupported_arg_names_skips_when_schema_is_none() {
    let args = json!({ "anything": "goes" });
    assert_eq!(unsupported_arg_names(None, &args), None);
}

#[test]
fn unsupported_arg_names_skips_when_schema_has_no_properties_object() {
    // Legacy/loose schema shape (no `properties` map at all) — nothing to
    // validate names against, so this must skip, not reject.
    let schema = json!({ "type": "object", "description": "legacy shape" });
    let args = json!({ "anything": "goes" });
    assert_eq!(unsupported_arg_names(Some(&schema), &args), None);
}

#[test]
fn unsupported_arg_names_empty_for_null_or_non_object_args() {
    let schema = json!({
        "type": "object",
        "properties": { "channel": {"type": "string"} }
    });
    assert_eq!(
        unsupported_arg_names(Some(&schema), &Value::Null),
        Some(vec![])
    );
    assert_eq!(
        unsupported_arg_names(Some(&schema), &json!("not an object")),
        Some(vec![])
    );
}

#[test]
fn compute_primary_array_path_finds_a_top_level_array_property() {
    let schema = json!({
        "type": "object",
        "properties": { "items": { "type": "array" }, "count": { "type": "integer" } }
    });
    assert_eq!(
        compute_primary_array_path(Some(&schema)),
        Some("items".to_string())
    );
}

#[test]
fn compute_primary_array_path_finds_a_nested_array_property() {
    // Gmail-shaped: the array lives two levels down, under `data.messages`.
    let schema = json!({
        "type": "object",
        "properties": {
            "data": {
                "type": "object",
                "properties": {
                    "messages": { "type": "array" },
                    "nextPageToken": { "type": "string" }
                }
            }
        }
    });
    assert_eq!(
        compute_primary_array_path(Some(&schema)),
        Some("data.messages".to_string())
    );
}

#[test]
fn compute_primary_array_path_none_when_absent_or_no_array_property() {
    assert_eq!(compute_primary_array_path(None), None);
    assert_eq!(
        compute_primary_array_path(Some(&json!({ "type": "object" }))),
        None
    );
    assert_eq!(
        compute_primary_array_path(Some(
            &json!({ "type": "object", "properties": { "id": { "type": "string" } } })
        )),
        None
    );
}
