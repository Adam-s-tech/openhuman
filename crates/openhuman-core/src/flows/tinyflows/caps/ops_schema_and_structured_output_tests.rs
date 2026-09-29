use super::*;

// ── resolve_completion_model raw/BYOK passthrough (issue #4598) ───────────
#[test]
fn resolve_completion_model_forwards_raw_byok_node_model_verbatim() {
    // A raw/BYOK id maps to the `chat` role, so the role resolves to the
    // default model — but the pinned id is what the user selected and must
    // be the model the completion runs on.
    assert_eq!(
        resolve_completion_model(Some("claude-opus-4"), "chat-v1".to_string()),
        "claude-opus-4"
    );
    assert_eq!(
        resolve_completion_model(Some("deepseek-v4-pro"), "chat-v1".to_string()),
        "deepseek-v4-pro"
    );
}

#[test]
fn resolve_completion_model_leaves_managed_tier_and_hint_node_models_untouched() {
    // Managed tiers and every `hint:*` alias keep the role-resolved model.
    assert_eq!(
        resolve_completion_model(Some("chat-v1"), "chat-v1".to_string()),
        "chat-v1"
    );
    assert_eq!(
        resolve_completion_model(Some("hint:reasoning"), "reasoning-v1".to_string()),
        "reasoning-v1"
    );
    assert_eq!(
        resolve_completion_model(Some("hint:garbage"), "reasoning-v1".to_string()),
        "reasoning-v1"
    );
    // No pinned model, or a whitespace-only pin, keeps the resolved default.
    assert_eq!(
        resolve_completion_model(None, "chat-v1".to_string()),
        "chat-v1"
    );
    assert_eq!(
        resolve_completion_model(Some("   "), "chat-v1".to_string()),
        "chat-v1"
    );
}

#[test]
fn crate_model_response_preserves_flow_completion_contract() {
    use tinyinference_llm::message::{AssistantMessage, ContentBlock};
    use tinyinference_llm::model::ModelResponse;
    use tinyinference_llm::tool::ToolCall;
    use tinyinference_llm::usage::Usage;

    let usage = Usage::new(11, 7);
    let response = ModelResponse {
        message: AssistantMessage {
            id: Some("msg-1".to_string()),
            content: vec![
                ContentBlock::Text("done".to_string()),
                ContentBlock::thinking("private chain"),
            ],
            tool_calls: vec![ToolCall {
                id: "call-1".to_string(),
                name: "lookup".to_string(),
                arguments: json!({"query": "weather"}),
                invalid: None,
            }],
            usage: Some(usage),
            origin: None,
        },
        usage: Some(usage),
        finish_reason: Some("tool_calls".to_string()),
        raw: crate::agent::tinyagents::model::merge_openhuman_usage_meta(None, 0.125, 128_000),
        resolved_model: None,
        continue_turn: None,
        served_from_cache: false,
        correlation: None,
        resolved_route: None,
    };

    let value = model_response_to_completion_value(&response);
    assert_eq!(value["text"], "done");
    assert_eq!(value["tool_calls"][0]["id"], "call-1");
    assert_eq!(value["tool_calls"][0]["name"], "lookup");
    assert_eq!(
        value["tool_calls"][0]["arguments"],
        r#"{"query":"weather"}"#
    );
    assert_eq!(value["usage"]["input_tokens"], 11);
    assert_eq!(value["usage"]["output_tokens"], 7);
    assert_eq!(value["usage"]["context_window"], 128_000);
    assert_eq!(value["usage"]["charged_amount_usd"], 0.125);
    assert_eq!(value["reasoning_content"], "private chain");
}

// ── build_agent_result improvements (issue #5151) ────────────────────

#[test]
fn build_agent_result_extracts_embedded_json_from_prose_text() {
    // When the agent's final text wraps JSON in prose without fence
    // blocks (e.g. the LLM explains the result before outputting the
    // data), build_agent_result must still extract the object rather than
    // falling back to {text, agent_ref} which kills the downstream
    // output_parser.
    let request = json!({
        "output_parser": {
            "schema": { "type": "object", "required": ["name"] }
        }
    });
    let result = build_agent_result(
        "agent-1",
        "The result is: { \"name\": \"Alice\", \"age\": 30 }",
        &request,
    );
    assert_eq!(result, json!({ "name": "Alice", "age": 30 }));
}

#[test]
fn build_agent_result_extracts_embedded_array_from_prose_text() {
    let request = json!({
        "output_parser": {
            "schema": { "type": "array" }
        }
    });
    let result = build_agent_result("agent-1", "Here is the list: [1, 2, 3]", &request);
    assert_eq!(result, json!([1, 2, 3]));
}

#[test]
fn structured_json_extraction_ignores_braces_inside_strings() {
    let text = r#"Result: {"note":"use } to close and \"quote\" safely","ok":true}"#;
    assert_eq!(
        extract_structured_json(text),
        Some(json!({"note": "use } to close and \"quote\" safely", "ok": true}))
    );
}

#[test]
fn structured_json_extraction_uses_fenced_then_balanced_fallbacks() {
    assert_eq!(
        extract_structured_json("preface\n```json\n{\"fenced\":true}\n```"),
        Some(json!({"fenced": true}))
    );
    assert_eq!(
        extract_structured_json("preface {\"embedded\":true} suffix"),
        Some(json!({"embedded": true}))
    );
}

#[test]
fn build_agent_result_falls_back_to_text_when_no_json_found_in_prose() {
    // Pure prose with no JSON-like content must still fall back to the
    // safe {text, agent_ref} shape.
    let request = json!({
        "output_parser": {
            "schema": { "type": "object", "required": ["name"] }
        }
    });
    let result = build_agent_result(
        "agent-1",
        "I searched for the information but could not find it.",
        &request,
    );
    assert_eq!(
        result,
        json!({ "text": "I searched for the information but could not find it.",
                "agent_ref": "agent-1" })
    );
}

#[test]
fn build_agent_result_prefers_fenced_json_over_balanced_brace_extraction() {
    // When both a fenced block and loose prose-with-JSON are present,
    // the fenced block wins (it's the canonical / better-specified
    // format).
    let request = json!({
        "output_parser": {
            "schema": { "type": "object" }
        }
    });
    let text =
        "Some text\n```json\n{\"from_fence\": true}\n```\nmore text { \"from_brace\": true }";
    let result = build_agent_result("agent-1", text, &request);
    assert_eq!(result, json!({ "from_fence": true }));
}
