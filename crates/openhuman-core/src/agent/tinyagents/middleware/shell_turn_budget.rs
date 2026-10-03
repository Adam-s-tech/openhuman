//! Shell turn budget stub (#6953).

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use tinyagents_harness::context::RunContext;
use tinyagents_harness::middleware::Middleware;
use tinyinference_llm::tool::ToolCall;
use tinytools::ToolResult;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ShellTimeoutClamp {
    pub requested: Option<u64>,
    pub clamped_secs: u64,
    pub remaining_secs: u64,
}

impl ShellTimeoutClamp {
    pub(crate) fn note(&self) -> String {
        String::new()
    }
}

pub(crate) fn clamp_shell_timeout(_requested: Option<u64>, _remaining: Duration) -> Option<ShellTimeoutClamp> {
    None
}

pub(crate) struct ShellTurnBudget;
impl ShellTurnBudget {
    pub(crate) fn new(_budget: Option<Duration>) -> Arc<Self> { Arc::new(Self) }
    pub(crate) fn clamp(self: &Arc<Self>) -> ShellTimeoutClampMiddleware { ShellTimeoutClampMiddleware }
    pub(crate) fn notes(self: &Arc<Self>) -> ShellTimeoutNoteMiddleware { ShellTimeoutNoteMiddleware }
}
pub(crate) struct ShellTimeoutClampMiddleware;
pub(crate) struct ShellTimeoutNoteMiddleware;
#[async_trait]
impl<C: Send + Sync> Middleware<(), C> for ShellTimeoutClampMiddleware {
    fn name(&self) -> &str { "shell_turn_budget" }
}
#[async_trait]
impl<C: Send + Sync> Middleware<(), C> for ShellTimeoutNoteMiddleware {
    fn name(&self) -> &str { "shell_turn_budget_note" }
}

#[cfg(test)]
#[path = "shell_turn_budget_tests.rs"]
mod tests;
