//! The process-global hook engine and the OpenHuman seams it is built with.
//!
//! The engine, `hooks.json` contract and process runner live in
//! `tinyagents_runtime::command_hooks`. This module supplies what is specific
//! to this host: the product name that decides discovery paths and the
//! `OPENHUMAN_*` hook environment, the user's home directory, the platform
//! shell, and the model behind `prompt` hooks.

use std::sync::{Arc, LazyLock};

use async_trait::async_trait;
use tinyagents_runtime::command_hooks::{HookEngine, HookEnvironment, PromptEvaluator};

/// Product name handed to the engine: `ProgramData\OpenHuman`,
/// `/Library/Application Support/OpenHuman`, `/etc/openhuman`, `.openhuman`,
/// and the `OPENHUMAN_*` hook variables all derive from it.
pub const PRODUCT_NAME: &str = "OpenHuman";

struct HostPromptEvaluator;

#[async_trait]
impl PromptEvaluator for HostPromptEvaluator {
    async fn evaluate(&self, instruction: &str, model: Option<&str>) -> Result<String, String> {
        super::prompt_eval::evaluate(instruction, model).await
    }
}

fn environment() -> HookEnvironment {
    HookEnvironment::new(PRODUCT_NAME, dirs::home_dir())
        .with_shell(crate::agent::platform_shell::build_tokio_command)
        .with_prompt_evaluator(Arc::new(HostPromptEvaluator))
}

static ENGINE: LazyLock<HookEngine> = LazyLock::new(|| HookEngine::new(environment()));

/// The process-global engine.
pub fn engine() -> &'static HookEngine {
    &ENGINE
}
