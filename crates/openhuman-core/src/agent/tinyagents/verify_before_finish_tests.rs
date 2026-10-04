use super::*;

use tinyagents_harness::limits::RunLimits;
use tinyagents_harness::middleware::{
    CapturedOutcomes, FinalCallWrapUpMiddleware, OutcomesUnavailable,
};
use tinyagents_harness::runtime::RunPolicy;
use tinyagents_harness::testkit::{FakeTool, ScriptedModel};
use tinyagents_harness::tinyinference_llm::message::Message;
use tinyagents_harness::tinyinference_llm::model::ModelResponse;
use tinyagents_harness::tinyinference_llm::tool::ToolCall;

fn activity(rounds: usize, tools: &[&str]) -> FinishActivity {
    FinishActivity {
        tool_rounds: rounds,
        tools_called: tools.iter().map(|t| (*t).to_string()).collect(),
    }
}

fn tool_round(id: &str, name: &str) -> ModelResponse {
    let mut response = ModelResponse::assistant(String::new());
    response.message.content = Vec::new();
    response.message.tool_calls = vec![ToolCall::new(id, name, serde_json::json!({}))];
    response.finish_reason = Some("tool_calls".to_string());
    response
}

/// Drive a harness with `install` applied, `rounds` tool rounds of `tool`,
/// then a draft and a checked answer. Returns how many check turns the run's
/// transcript carries and its final text.
async fn drive(rounds: usize, tool: &str, subagent: bool, agent: Option<&str>) -> (usize, String) {
    drive_with(rounds, tool, subagent, agent, WrapUp::Absent, 50).await
}

/// No captured outcomes: the wrap-up under test only needs its budget notice.
struct NoOutcomes;

impl CapturedOutcomes for NoOutcomes {
    fn content_for(&self, _: &str) -> Result<Option<String>, OutcomesUnavailable> {
        Ok(None)
    }
}

/// How the wrap-up middleware reaches the check in a driven run.
#[derive(Clone, Copy)]
enum WrapUp {
    /// No wrap-up middleware at all.
    Absent,
    /// Installed with a budget notice, and handed to `install` (the wiring the
    /// harness assembly does with `wrap_up_fired`).
    Linked,
    /// Installed with a budget notice but NOT handed to `install`.
    Unlinked,
}

async fn drive_with(
    rounds: usize,
    tool: &str,
    subagent: bool,
    agent: Option<&str>,
    wrap_up: WrapUp,
    max_model_calls: u64,
) -> (usize, String) {
    let mut responses: Vec<ModelResponse> = (0..rounds)
        .map(|i| tool_round(&format!("c{i}"), tool))
        .collect();
    responses.push(ModelResponse::assistant("draft".to_string()));
    responses.push(ModelResponse::assistant("checked".to_string()));
    let mut harness: AgentHarness<()> = AgentHarness::new();
    harness.register_model("mock", Arc::new(ScriptedModel::new(responses)));
    harness.register_tool(Arc::new(FakeTool::returning("lookup", "ok")));
    harness.register_tool(Arc::new(FakeTool::returning("todo", "ok")));
    harness.with_policy(RunPolicy {
        limits: RunLimits::default()
            .with_max_model_calls(max_model_calls)
            .with_max_tool_calls(50),
        ..RunPolicy::default()
    });
    let wrap_up_mw = (!matches!(wrap_up, WrapUp::Absent)).then(|| {
        Arc::new(
            FinalCallWrapUpMiddleware::new("CONCLUDE", "WRITE", Arc::new(NoOutcomes), 0)
                .with_budget_notice([0.5]),
        )
    });
    if let Some(mw) = &wrap_up_mw {
        harness.push_middleware(mw.clone());
    }
    let linked = matches!(wrap_up, WrapUp::Linked)
        .then(|| wrap_up_mw.as_ref())
        .flatten();
    install(&mut harness, subagent, agent, linked);
    let run = harness
        .invoke_default(&(), vec![Message::user("do the task")])
        .await
        .expect("run succeeds");
    let checks = run
        .messages
        .iter()
        .filter(|m| matches!(m, Message::User(_)) && m.text().contains(CHECK_MARKER))
        .count();
    (checks, run.text().unwrap_or_default())
}

#[test]
fn applies_to_root_orchestrator_turns_only() {
    assert!(applies(false, Some("orchestrator")));
    assert!(!applies(true, Some("orchestrator")), "sub-agents never");
    assert!(!applies(false, Some("welcome")), "other root agents never");
    assert!(
        !applies(false, None),
        "a turn without an agent identity never"
    );
}

#[test]
fn triggers_on_enough_tool_rounds_or_a_todo_list() {
    assert!(!should_check(&activity(MIN_TOOL_ROUNDS - 1, &["shell"])));
    assert!(should_check(&activity(MIN_TOOL_ROUNDS, &["shell"])));
    assert!(should_check(&activity(1, &[TODO_TOOL])));
    assert!(!should_check(&activity(0, &[])));
}

#[test]
fn check_is_a_harness_instruction_that_names_the_spec_rules() {
    let check = check_message();
    assert!(check.starts_with("<harness_instruction>"));
    assert!(check.contains(CHECK_MARKER));
    for needle in ["original request", "literal rule", "derived from the spec"] {
        assert!(check.contains(needle), "check must mention `{needle}`");
    }
}

#[tokio::test]
async fn installed_root_orchestrator_turn_checks_once_after_five_rounds() {
    let (checks, text) = drive(MIN_TOOL_ROUNDS, "lookup", false, Some("orchestrator")).await;
    assert_eq!(checks, 1);
    assert_eq!(text, "checked");
}

#[tokio::test]
async fn installed_turn_with_a_todo_list_checks_after_one_round() {
    let (checks, text) = drive(1, TODO_TOOL, false, Some("orchestrator")).await;
    assert_eq!(checks, 1);
    assert_eq!(text, "checked");
}

#[tokio::test]
async fn short_turn_is_not_checked() {
    let (checks, text) = drive(2, "lookup", false, Some("orchestrator")).await;
    assert_eq!(checks, 0);
    assert_eq!(text, "draft");
}

#[tokio::test]
async fn subagent_turn_is_not_checked() {
    let (checks, text) = drive(MIN_TOOL_ROUNDS, "lookup", true, Some("orchestrator")).await;
    assert_eq!(checks, 0);
    assert_eq!(text, "draft");
}

/// tinyagents#301: once the wrap-up has announced a budget notice the check
/// stays quiet. This only holds when `install` is handed the SAME `Arc` that is
/// installed as the wrap-up middleware, which is what the harness assembly does.
#[tokio::test]
async fn a_linked_wrap_up_budget_notice_silences_the_check() {
    // 5 tool rounds + the draft = 6 calls of a budget of 8: the 0.5 notice
    // fires on the 4th, before the draft.
    let rounds = MIN_TOOL_ROUNDS;
    let (checks, text) = drive_with(
        rounds,
        "lookup",
        false,
        Some("orchestrator"),
        WrapUp::Linked,
        8,
    )
    .await;
    assert_eq!(checks, 0, "a budget notice said finish; no re-verify turn");
    assert_eq!(text, "draft");
}

#[tokio::test]
async fn an_unlinked_wrap_up_does_not_silence_the_check() {
    let (checks, _) = drive_with(
        MIN_TOOL_ROUNDS,
        "lookup",
        false,
        Some("orchestrator"),
        WrapUp::Unlinked,
        8,
    )
    .await;
    assert_eq!(
        checks, 1,
        "without the shared Arc the check cannot see the notice"
    );
}
