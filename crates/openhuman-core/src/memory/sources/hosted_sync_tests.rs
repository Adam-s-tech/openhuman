//! The host's sync of local sources into a driver's sink: which items a run
//! reads, what it sends, and the budgets it keeps.

use std::sync::Mutex;

use async_trait::async_trait;

use super::*;
use crate::memory::api::provider::types::IngestOutcome;

/// One `accept_source_items` call: source id, source kind, items, taint.
type Batch = (String, String, Vec<SinkItem>, MemoryTaint);

/// A sink that records every batch and writes each item it is handed.
#[derive(Default)]
struct RecordingSink {
    batches: Mutex<Vec<Batch>>,
}

impl RecordingSink {
    fn items(&self) -> Vec<SinkItem> {
        self.batches
            .lock()
            .expect("batches")
            .iter()
            .flat_map(|(_, _, items, _)| items.clone())
            .collect()
    }
}

#[async_trait]
impl MemorySourceSink for RecordingSink {
    async fn accept_source_items(
        &self,
        source_id: &str,
        source_kind: &str,
        items: Vec<SinkItem>,
        taint: MemoryTaint,
    ) -> Result<IngestOutcome, MemoryError> {
        let written = items.len() as u32;
        self.batches.lock().expect("batches").push((
            source_id.to_string(),
            source_kind.to_string(),
            items,
            taint,
        ));
        Ok(IngestOutcome {
            written,
            ..IngestOutcome::default()
        })
    }

    async fn forget_source(&self, _source_id: &str) -> Result<u64, MemoryError> {
        Ok(0)
    }
}

fn folder_entry(path: &std::path::Path) -> MemorySourceEntry {
    serde_json::from_value(serde_json::json!({
        "id": "notes",
        "kind": "folder",
        "label": "Notes",
        "enabled": true,
        "path": path.to_string_lossy(),
    }))
    .expect("a folder source")
}

fn listed(id: &str, updated_at_ms: Option<i64>) -> SourceItem {
    SourceItem {
        id: id.to_string(),
        title: id.to_string(),
        updated_at_ms,
    }
}

fn config(dir: &std::path::Path) -> Config {
    Config {
        workspace_dir: dir.to_path_buf(),
        ..Config::default()
    }
}

#[test]
fn the_host_syncs_local_kinds_only() {
    for kind in [
        SourceKind::Folder,
        SourceKind::GithubRepo,
        SourceKind::RssFeed,
        SourceKind::WebPage,
    ] {
        assert!(serves(&kind), "{kind:?}");
    }
    for kind in [
        SourceKind::Composio,
        SourceKind::Conversation,
        SourceKind::TwitterQuery,
    ] {
        assert!(!serves(&kind), "{kind:?}");
    }
    assert_eq!(taint_of(&SourceKind::Folder), MemoryTaint::Internal);
    assert_eq!(taint_of(&SourceKind::RssFeed), MemoryTaint::ExternalSync);
}

#[test]
fn a_run_reads_the_newest_items_within_its_depth_and_cap() {
    let day = 86_400_000;
    let now = 100 * day;
    let mut entry = folder_entry(std::path::Path::new("."));
    let items = || {
        vec![
            listed("old", Some(now - 40 * day)),
            listed("new", Some(now - day)),
            listed("undated", None),
            listed("mid", Some(now - 10 * day)),
        ]
    };
    let ids = |items: Vec<SourceItem>| items.into_iter().map(|i| i.id).collect::<Vec<_>>();
    assert_eq!(
        ids(chosen(&entry, items(), now)),
        ["new", "mid", "old", "undated"]
    );
    entry.sync_depth_days = Some(30);
    assert_eq!(
        ids(chosen(&entry, items(), now)),
        ["new", "mid", "undated"],
        "an item with no date is kept"
    );
    entry.max_items = Some(1);
    assert_eq!(ids(chosen(&entry, items(), now)), ["new"]);
}

#[tokio::test]
async fn a_folder_is_read_and_sent_to_the_sink() {
    let dir = tempfile::tempdir().expect("tempdir");
    let folder = dir.path().join("notes");
    std::fs::create_dir_all(&folder).expect("folder");
    std::fs::write(folder.join("a.md"), "# Tea\n\nOolong is best.").expect("a");
    std::fs::write(folder.join("b.md"), "   ").expect("blank");
    std::fs::write(folder.join("skip.txt"), "not markdown").expect("txt");
    let sink = RecordingSink::default();
    let outcome = run(&config(dir.path()), &folder_entry(&folder), &sink)
        .await
        .expect("run");
    assert_eq!(outcome.records_ingested, 1);
    assert!(!outcome.more_pending);
    let batches = sink.batches.lock().expect("batches");
    assert_eq!(batches.len(), 1);
    let (source_id, kind, items, taint) = &batches[0];
    assert_eq!((source_id.as_str(), kind.as_str()), ("notes", "folder"));
    assert_eq!(*taint, MemoryTaint::Internal);
    assert_eq!(items.len(), 1, "a blank file is not sent");
    assert!(items[0].content.contains("Oolong is best."));
    assert_eq!(items[0].mime.as_deref(), Some("text/markdown"));
}

#[tokio::test]
async fn a_token_budget_stops_the_run_with_more_pending() {
    let dir = tempfile::tempdir().expect("tempdir");
    let folder = dir.path().join("notes");
    std::fs::create_dir_all(&folder).expect("folder");
    for name in ["a", "b", "c"] {
        std::fs::write(folder.join(format!("{name}.md")), "x".repeat(400)).expect("file");
    }
    let mut entry = folder_entry(&folder);
    entry.max_tokens_per_sync = Some(150);
    let sink = RecordingSink::default();
    let outcome = run(&config(dir.path()), &entry, &sink).await.expect("run");
    assert_eq!(
        sink.items().len(),
        1,
        "600 characters fit one 400-character file"
    );
    assert!(outcome.more_pending);
}

#[tokio::test]
async fn a_missing_folder_or_unserved_kind_fails_the_run() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sink = RecordingSink::default();
    let missing = run(
        &config(dir.path()),
        &folder_entry(&dir.path().join("absent")),
        &sink,
    )
    .await;
    assert!(
        matches!(missing, Err(MemoryError::Backend(_))),
        "{missing:?}"
    );
    let mut chats = folder_entry(dir.path());
    chats.kind = SourceKind::Conversation;
    let refused = run(&config(dir.path()), &chats, &sink).await;
    assert!(
        matches!(refused, Err(MemoryError::Unsupported { .. })),
        "{refused:?}"
    );
    assert!(sink.items().is_empty());
}
