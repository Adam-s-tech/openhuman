use super::*;
use crate::security::{AutonomyLevel, SecurityPolicy};
use serde_json::json;
use std::sync::Arc;
use tinyagents_harness::artifacts::tool_results::apply_per_result_persistence;
use tinytools::Tool;

/// End to end through OpenHuman's wiring: the real `sanitize_text` redacts the
/// stored body and the preview, and the real `file_read` opens the artifact at
/// the path the envelope names.
#[tokio::test]
async fn threshold_persists_preview_and_readable_file() {
    let tmp = tempfile::tempdir().unwrap();
    let store = new_tool_result_store(tmp.path().to_path_buf(), "session/one");
    let raw = format!(
        "{} {}",
        "x".repeat(4096),
        "ghp_abcdefghijklmnopqrstuvwxyz123456"
    );

    let (out, outcome) = apply_per_result_persistence(
        raw.clone(),
        None,
        Some(&store),
        "shell",
        Some("call-1"),
        1024,
    )
    .await;

    assert!(outcome.persisted);
    assert!(out.contains("artifact_path: artifacts/tool-results/session_one/shell/call-1.txt"));
    assert!(out.contains("original_bytes:"));
    assert!(out.contains("[preview]"));
    assert!(!out.contains("ghp_abcdefghijklmnopqrstuvwxyz123456"));

    let policy = Arc::new(SecurityPolicy {
        autonomy: AutonomyLevel::ReadOnly,
        action_dir: tmp.path().to_path_buf(),
        workspace_dir: tmp.path().to_path_buf(),
        ..SecurityPolicy::default()
    });
    let reader = FileReadTool::new(policy);
    let read = reader
        .execute(json!({"path": "artifacts/tool-results/session_one/shell/call-1.txt"}))
        .await
        .unwrap();
    assert!(!read.is_error, "{}", read.output());
    assert!(read.output().contains("xxxx"));
    assert!(!read
        .output()
        .contains("ghp_abcdefghijklmnopqrstuvwxyz123456"));
}

/// The vocabulary passed to the crate is OpenHuman's: `file_read` reads,
/// `use_skill` is the only wrapper followed.
#[test]
fn read_targets_use_openhumans_tool_names() {
    let path = "artifacts/tool-results/s/shell/c.txt";
    assert!(artifact_read_target("file_read", &json!({"path": path})).is_some());
    assert!(artifact_read_target(
        "use_skill",
        &json!({"skill": "files", "tool": "file_read", "args": {"path": path, "offset": 3}})
    )
    .is_some_and(|read| read.offset == 3));
    assert!(artifact_read_target("glob", &json!({"path": path})).is_none());
}

/// A continuation names `file_read` and the offset, in the wire format the
/// model has always seen.
#[test]
fn a_page_names_file_read_as_the_continuation() {
    let read = ArtifactRead {
        path: "artifacts/tool-results/s/shell/c.txt".to_string(),
        offset: 0,
    };
    let page = page_artifact_read("y".repeat(5_000), &read, 1_000);
    assert!(
        page.contains("Continue with file_read {\"path\":\"artifacts/tool-results/s/shell/c.txt\",\"offset\":"),
        "{page}"
    );
}
