//! Bound direct web research so a run that keeps finding more leads still answers.

use std::sync::atomic::{AtomicUsize, Ordering};

use async_trait::async_trait;
use tinyagents_harness::context::RunContext;
use tinyagents_harness::error::Result as TaResult;
use tinyagents_harness::middleware::{Middleware, ToolInvocationIdentity};
use tinyinference_llm::message::Message;
use tinyinference_llm::model::{ModelRequest, ToolChoice};
use tinytools::ToolResult;

use crate::agent::tinyagents::host::OpenHumanRunContext;

/// Direct web reads allowed before the next model call must conclude the turn.
/// This bounds the master agent's exploratory search while leaving room to read
/// primary sources; broad research belongs in one deep `web_answer_tool` call or
/// a batched `web_contents_tool` read, each of which counts once.
pub(super) const DIRECT_WEB_READ_LIMIT: usize = 8;

/// The instruction appended when the budget is spent and the tools are
/// withdrawn. It says outright that tools are gone and a call will not run:
/// told only to "answer", a native model with a transcript full of tool calls
/// and no tool channel writes its next call as plain-text markup (DeepSeek V4's
/// `<｜DSML｜invoke …>`), which then stood as the turn's answer. Replaying a
/// bench request that hit this budget, the old wording leaked a call 6 times in
/// 8 and this wording 0 times in 8. The harness also withholds and re-prompts
/// any call that still arrives (tinyagents `TextRecovery::withholding`).
pub(crate) const RESEARCH_CLOSE_INSTRUCTION: &str = "The direct web research budget for this turn is exhausted, and tools are no longer available for this reply: any tool call you write now will not run. Answer the user's latest request now in plain text, using only the results already available. State any remaining uncertainty. Do not search again, repeat a page fetch, or merely describe what you plan to read.";

#[derive(Default)]
pub(crate) struct ResearchBudgetMiddleware {
    completed_reads: AtomicUsize,
}

impl ResearchBudgetMiddleware {
    pub(crate) fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl Middleware<(), OpenHumanRunContext> for ResearchBudgetMiddleware {
    fn name(&self) -> &str {
        "research_budget"
    }

    async fn after_tool(
        &self,
        _ctx: &mut RunContext<OpenHumanRunContext>,
        _state: &(),
        invocation: &ToolInvocationIdentity,
        _result: &mut ToolResult,
    ) -> TaResult<()> {
        if matches!(
            invocation.tool_name(),
            "web_search_tool" | "web_answer_tool" | "web_contents_tool" | "web_fetch"
        ) {
            self.completed_reads.fetch_add(1, Ordering::Relaxed);
        }
        Ok(())
    }

    async fn before_model(
        &self,
        _ctx: &mut RunContext<OpenHumanRunContext>,
        _state: &(),
        request: &mut ModelRequest,
    ) -> TaResult<()> {
        if self.completed_reads.load(Ordering::Relaxed) < DIRECT_WEB_READ_LIMIT {
            return Ok(());
        }
        tracing::info!(
            web_reads = self.completed_reads.load(Ordering::Relaxed),
            "[tinyagents::mw] direct web research budget reached; concluding turn"
        );
        request.tools.clear();
        request.tool_choice = ToolChoice::None;
        request
            .messages
            .push(Message::user(RESEARCH_CLOSE_INSTRUCTION.to_string()));
        Ok(())
    }
}
