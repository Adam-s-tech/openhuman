use super::*;

fn tool() -> MemoryTool {
    MemoryTool::new(
        Arc::new(Config::default()),
        Arc::new(SecurityPolicy::default()),
    )
}

#[test]
fn every_member_is_hidden_so_the_collapse_actually_saves_something() {
    // `all_actions`, not `actions`: capability filtering could hide a
    // still-advertised member from this check in some environments, and
    // the property being pinned holds regardless of capabilities.
    for entry in tool().all_actions() {
        assert_eq!(
            entry.tool.exposure(),
            ToolExposure::Hidden,
            "`{}` is still advertised alongside the collapsed `memory` tool",
            entry.tool.name()
        );
    }
}

#[test]
fn the_schema_advertises_every_action() {
    let schema = tool().parameters_schema();
    let listed = schema["properties"]["action"]["enum"]
        .as_array()
        .expect("enum")
        .len();
    assert_eq!(listed, 4);
}

#[test]
fn each_action_resolves_to_exactly_its_members_level() {
    // The real contract, and the one that keeps collapsing honest: whatever
    // a member declares, the collapsed tool reports for that action.
    //
    // Note this family currently declares one level between them (see the
    // module docs): `memory_store` and `memory_forget` never override
    // `permission_level`, so they inherit the `ReadOnly` default. That is
    // pre-existing and deliberately not changed here — raising them is a
    // behaviour change to the approval gate, not a token optimisation. An
    // inequality assertion would therefore be asserting a bug.
    let tool = tool();
    for entry in tool.actions() {
        let args = serde_json::json!({"action": entry.action});
        assert_eq!(
            tool.permission_level_with_args(&args),
            entry.tool.permission_level_with_args(&args),
            "action `{}` must report what `{}` reports",
            entry.action,
            entry.tool.name()
        );
    }
}

#[test]
fn collapsing_never_lowers_the_argument_free_level() {
    // The safety property that does not depend on what the members happen
    // to declare today: a caller that cannot pass arguments is never told
    // a level below any member's.
    let tool = tool();
    let floor = tool.permission_level();
    for entry in tool.actions() {
        assert!(
            entry.tool.permission_level() <= floor,
            "`{}` requires more than the collapsed tool advertises",
            entry.tool.name()
        );
    }
}

#[test]
fn an_unknown_action_falls_back_to_the_strictest_level() {
    let tool = tool();
    assert_eq!(
        tool.permission_level_with_args(&serde_json::json!({"action": "nope"})),
        tool.permission_level()
    );
}

#[tokio::test]
async fn an_unknown_action_is_an_error_result_naming_the_valid_ones() {
    let result = tool()
        .execute(serde_json::json!({"action": "recal", "text": "x"}))
        .await
        .expect("dispatch does not fail the call");
    assert!(result.is_error);
    let text = format!("{result:?}");
    assert!(text.contains("recal"));
    assert!(text.contains("ask|keyword_search|learn|forget"));
}

#[test]
fn the_memory_tree_tool_is_not_a_member() {
    // Pinning the decision in the module docs: `memory_tree` dispatches on
    // its own `mode`, and folding it in would make this two-level.
    assert!(
        !tool()
            .all_actions()
            .iter()
            .any(|e| e.tool.name() == "memory_tree"),
        "memory_tree stays a separate tool"
    );
}

#[test]
fn each_action_maps_text_onto_its_members_arguments() {
    let ask = member_args(ACTION_ASK, &serde_json::json!({"text": " who is Ana? "})).unwrap();
    assert_eq!(ask["query"], "who is Ana?");
    assert!(
        ask["namespace"].is_string(),
        "hybrid search needs a namespace"
    );

    let keywords = member_args(
        ACTION_KEYWORD_SEARCH,
        &serde_json::json!({"text": "ana", "limit": 3}),
    )
    .unwrap();
    assert_eq!(keywords["query"], "ana");
    assert_eq!(keywords["limit"], 3);

    let learn = member_args(
        ACTION_LEARN,
        &serde_json::json!({"text": "Ana prefers email", "limit": 9}),
    )
    .unwrap();
    assert_eq!(learn["content"], "Ana prefers email");
    assert!(learn.get("limit").is_none(), "a write takes no limit");
}

#[test]
fn empty_text_is_refused() {
    assert!(member_args(ACTION_ASK, &serde_json::json!({"text": "  "})).is_err());
    assert!(member_args(ACTION_LEARN, &serde_json::json!({})).is_err());
}

#[test]
fn forget_takes_the_key_as_text_and_defaults_the_namespace() {
    let args = member_args(
        ACTION_FORGET,
        &serde_json::json!({"text": " ana_email_pref "}),
    )
    .unwrap();
    assert_eq!(args["key"], "ana_email_pref");
    assert!(args["namespace"].is_string(), "forget requires a namespace");
    assert!(args.get("query").is_none() && args.get("content").is_none());
}

#[test]
fn advertised_actions_reads_the_schema_enum_and_ignores_other_tools() {
    let memory = tool();
    let listed = advertised_actions(&memory);
    assert!(
        listed.iter().any(|a| a == ACTION_LEARN)
            == memory.actions().iter().any(|e| e.action == ACTION_LEARN)
    );
    assert!(
        listed.len() <= 4 && !listed.is_empty() || memory.actions().is_empty(),
        "{listed:?}"
    );
    // Any tool that is not the collapsed `memory` tool advertises no actions,
    // so a name check elsewhere cannot mistake it for one.
    assert!(advertised_actions(&MemoryRecallTool::new()).is_empty());
}
