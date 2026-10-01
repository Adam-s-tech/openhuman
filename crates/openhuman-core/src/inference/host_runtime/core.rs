use std::sync::Arc;

use crate::config::Config;

use super::service::LocalAiService;

static LOCAL_AI: once_cell::sync::OnceCell<Arc<LocalAiService>> = once_cell::sync::OnceCell::new();

pub fn global(config: &Config) -> Arc<LocalAiService> {
    let runtime = crate::inference::local_runtime_config(config);
    LOCAL_AI
        .get_or_init(|| Arc::new(LocalAiService::new(&runtime)))
        .clone()
}

/// Like [`global`] but returns `None` instead of initialising the singleton.
///
/// Useful from shutdown paths where lazy-creating the service just to call a
/// no-op cleanup would be wasteful — if local AI was never used in this
/// process, there's nothing to clean up.
pub fn try_global() -> Option<Arc<LocalAiService>> {
    LOCAL_AI.get().cloned()
}

#[cfg(test)]
#[path = "core_tests.rs"]
mod tests;
