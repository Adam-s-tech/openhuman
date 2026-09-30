//! OpenHuman's wiring of per-tool-result artifact persistence.
//!
//! The mechanics — the store, the `[tool_result_preview]` envelope, the
//! per-result and aggregate budgets, paged artifact reads — live in
//! [`tinyagents_harness::artifacts::tool_results`]. What stays here is what only
//! the host can decide:
//!
//! * **the redactor**: the same `sanitize_text` pass that scrubs a worker
//!   artifact ([`SanitizingRedactor`]), because an artifact on disk is exactly as
//!   readable as the tool result it replaces;
//! * **the tool vocabulary**: `file_read` opens an artifact and `use_skill` is the
//!   one wrapper whose result *is* the wrapped tool's result;
//! * **the read limit**: the largest body `file_read` will open
//!   ([`FileReadTool::MAX_FILE_SIZE_BYTES`]).

use serde_json::Value;
use tinyagents_harness::artifacts::tool_results::{self, ArtifactRead, ToolResultArtifactStore};

use crate::agent::harness::artifact_offload::{SanitizingRedactor, READ_TOOL};
use tinytools_std::filesystem::FileReadTool;

/// Namespace of the artifact index in the run's store registry.
pub(crate) const TINYAGENTS_TOOL_RESULT_ARTIFACT_STORE: &str = "openhuman_tool_result_artifacts";

/// A store rooted at `action_dir` for one session, wired to OpenHuman's
/// redactor, read tool and read limit.
pub(crate) fn new_tool_result_store(
    action_dir: std::path::PathBuf,
    session_key: impl Into<String>,
) -> ToolResultArtifactStore {
    ToolResultArtifactStore::new(
        action_dir,
        session_key,
        std::sync::Arc::new(SanitizingRedactor),
        READ_TOOL,
        FileReadTool::MAX_FILE_SIZE_BYTES,
    )
}

/// The artifact a tool call reads, if any — `file_read`, possibly wrapped in
/// `use_skill` (#6284).
pub(crate) fn artifact_read_target(tool_name: &str, args: &Value) -> Option<ArtifactRead> {
    tool_results::artifact_read_target(
        tool_name,
        args,
        READ_TOOL,
        tinyagents_harness::tool::packs::USE_SKILL,
    )
}

/// Bound one page of an artifact read to `budget_bytes`.
pub(crate) fn page_artifact_read(
    content: String,
    read: &ArtifactRead,
    budget_bytes: usize,
) -> String {
    tool_results::page_artifact_read(content, read, budget_bytes, READ_TOOL)
}

#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;
