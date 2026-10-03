//! Skeleton.

use std::sync::Arc;

use tinyagents_harness::middleware::FinishActivity;
use tinyagents_harness::runtime::AgentHarness;

pub(super) const MIN_TOOL_ROUNDS: usize = 5;
pub(super) const TODO_TOOL: &str = "todo";
pub(super) const CHECK_MARKER: &str = "Before finishing";

pub(super) fn applies(_subagent: bool, _agent: Option<&str>) -> bool {
    false
}

pub(super) fn should_check(_activity: &FinishActivity) -> bool {
    false
}

pub(super) fn check_message() -> String {
    String::new()
}

pub(super) fn install<C: Send + Sync + 'static>(
    _harness: &mut AgentHarness<(), C>,
    _subagent: bool,
    _agent: Option<&str>,
) {
    let _ = Arc::new(());
}

#[cfg(test)]
#[path = "verify_before_finish_tests.rs"]
mod tests;
