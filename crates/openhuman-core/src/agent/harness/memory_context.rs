use super::memory_context_safety::{is_potentially_untrusted, wrap_untrusted_for_agent};
use crate::memory::Memory;
use crate::util::provenance_tag;
use std::collections::HashSet;
use std::fmt::Write;

/// Maximum number of `[Cross-chat context]` lines surfaced into the
/// working prompt. Tight cap on purpose: cross-chat hits are a recovery
/// signal for "I told you in another window" continuity, not a dump of
/// unrelated chats. See issue #1505.
pub(crate) const CROSS_CHAT_LIMIT: usize = 3;

/// Trim a cross-chat snippet to a bounded preview without panicking on
/// UTF-8 codepoint boundaries. Reuses the project-wide ellipsis helper
/// so the suffix accounting stays consistent with other prompt blocks.
fn shorten_for_cross_chat(content: &str) -> String {
    if content.chars().count() > CROSS_CHAT_SNIPPET_CHARS {
        crate::util::truncate_with_ellipsis(content, CROSS_CHAT_SNIPPET_CHARS)
    } else {
        content.to_string()
    }
}

/// Build context preamble by searching memory for relevant entries.
/// Entries with a hybrid score below `min_relevance_score` are dropped to
/// prevent unrelated memories from bleeding into the conversation.
pub(crate) async fn build_context(
    mem: &dyn Memory,
    user_msg: &str,
    min_relevance_score: f64,
) -> String {
    build_context_for_thread(mem, user_msg, min_relevance_score, None).await
}

#[cfg(test)]
#[path = "memory_context_tests.rs"]
mod tests;
