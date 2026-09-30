//! User-facing copy tied to a recognised response shape. The string-level
//! predicates themselves live in `tinyinference_llm::failure`.

/// User-facing copy for a poisoned-history 400 (orphaned tool message). The
/// de-poison guard (`run_task.rs`) has already evicted the offending warm
/// session by the time this is shown, so "send it again" is literally true.
pub(crate) fn malformed_history_user_message() -> &'static str {
    "We hit a temporary glitch in this conversation — we've cleared it. \
     Please send your message again."
}
