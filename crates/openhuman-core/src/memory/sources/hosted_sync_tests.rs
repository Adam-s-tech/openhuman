//! The host's sync of local sources into a driver's sink: which items a run
//! reads, what it sends, the budgets it keeps, and how the next run carries on
//! from what the sink accepted.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::{Duration, SystemTime};

use async_trait::async_trait;

use super::*;
use crate::memory::api::provider::types::IngestOutcome;

/// One `accept_source_items` call: source id, source kind, items, taint.
type Batch = (String, String, Vec<SinkItem>, MemoryTaint);

/// A sink that records every batch and writes each item it is handed, or
/// refuses the call numbered `refuse_call` (1-based).
#[derive(Default)]
struct RecordingSink {
    batches: Mutex<Vec<Batch>>,
    calls: AtomicUsize,
    refuse_call: Option<usize>,
}

impl RecordingSink {
    fn refusing(call: usize) -> Self {
        Self {
            refuse_call: Some(call),
            ..Self::default()
        }
    }

    fn items(&self) -> Vec<SinkItem> {
        self.batches
            .lock()
            .expect("batches")
            .iter()
            .flat_map(|(_, _, items, _)| items.clone())
            .collect()
    }

    fn item_ids(&self) -> Vec<String> {
        let mut ids: Vec<String> = self.items().into_iter().map(|item| item.item_id).collect();
        ids.sort();
        ids
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
        let call = self.calls.fetch_add(1, Ordering::SeqCst) + 1;
        if self.refuse_call == Some(call) {
            return Err(MemoryError::Unavailable("memory API unavailable".into()));
        }
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

/// A folder of `names`, each file `size` characters long, with distinct
/// modified times so the newest-first order is fixed: the first name is the
/// oldest.
fn folder_of(dir: &std::path::Path, names: &[&str], size: usize) -> std::path::PathBuf {
    let folder = dir.join("notes");
    std::fs::create_dir_all(&folder).expect("folder");
    let base = SystemTime::now() - Duration::from_secs(3_600);
    for (i, name) in names.iter().enumerate() {
        write_at(
            &folder.join(format!("{name}.md")),
            &format!("{name}{}", "x".repeat(size.saturating_sub(name.len()))),
            base + Duration::from_secs(i as u64 * 60),
        );
    }
    folder
}

fn write_at(path: &std::path::Path, text: &str, modified: SystemTime) {
    std::fs::write(path, text).expect("write");
    std::fs::File::options()
        .write(true)
        .open(path)
        .expect("open")
        .set_modified(modified)
        .expect("set the modified time");
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
fn a_run_considers_the_newest_items_within_its_depth() {
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

/// What the sink accepted is not sent again; a changed file is.
#[tokio::test]
async fn only_new_or_changed_items_are_sent_again() {
    let dir = tempfile::tempdir().expect("tempdir");
    let folder = folder_of(dir.path(), &["a", "b"], 40);
    let entry = folder_entry(&folder);
    let config = config(dir.path());
    let sink = RecordingSink::default();
    run(&config, &entry, &sink).await.expect("first");
    assert_eq!(sink.item_ids(), ["a.md", "b.md"]);

    let again = RecordingSink::default();
    let outcome = run(&config, &entry, &again).await.expect("unchanged");
    assert!(
        again.items().is_empty(),
        "an unchanged folder sends nothing"
    );
    assert_eq!(outcome.records_ingested, 0);

    write_at(&folder.join("b.md"), "b, rewritten", SystemTime::now());
    let changed = RecordingSink::default();
    run(&config, &entry, &changed).await.expect("changed");
    assert_eq!(changed.item_ids(), ["b.md"]);
}

/// A run stopped by its budget is picked up where it stopped, not restarted
/// from the newest items, so a source larger than its budget still syncs.
#[tokio::test]
async fn a_budget_limited_sync_carries_on_across_runs() {
    let dir = tempfile::tempdir().expect("tempdir");
    let folder = folder_of(dir.path(), &["a", "b", "c"], 400);
    let mut entry = folder_entry(&folder);
    // 600 characters: one 400-character file a run.
    entry.max_tokens_per_sync = Some(150);
    let config = config(dir.path());
    let mut sent = Vec::new();
    for _ in 0..3 {
        let sink = RecordingSink::default();
        let outcome = run(&config, &entry, &sink).await.expect("run");
        assert_eq!(sink.items().len(), 1);
        sent.extend(sink.item_ids());
        if sent.len() < 3 {
            assert!(outcome.more_pending);
        }
    }
    assert_eq!(sent, ["c.md", "b.md", "a.md"], "newest first, each once");
    let last = RecordingSink::default();
    let outcome = run(&config, &entry, &last).await.expect("done");
    assert!(last.items().is_empty());
    assert!(!outcome.more_pending);
}

/// `max_items` caps what one run sends, and the next run continues.
#[tokio::test]
async fn max_items_caps_each_run_and_the_next_one_continues() {
    let dir = tempfile::tempdir().expect("tempdir");
    let folder = folder_of(dir.path(), &["a", "b", "c"], 20);
    let mut entry = folder_entry(&folder);
    entry.max_items = Some(2);
    let config = config(dir.path());
    let first = RecordingSink::default();
    let outcome = run(&config, &entry, &first).await.expect("first");
    assert_eq!(first.item_ids(), ["b.md", "c.md"]);
    assert!(outcome.more_pending);
    let second = RecordingSink::default();
    run(&config, &entry, &second).await.expect("second");
    assert_eq!(second.item_ids(), ["a.md"]);
}

/// A refused batch fails the run, keeps what earlier batches wrote, and is
/// sent again next time; the earlier batches are not.
#[tokio::test]
async fn a_refused_batch_keeps_earlier_progress_and_is_retried() {
    let dir = tempfile::tempdir().expect("tempdir");
    let names: Vec<String> = (0..30).map(|i| format!("n{i:02}")).collect();
    let names: Vec<&str> = names.iter().map(String::as_str).collect();
    let folder = folder_of(dir.path(), &names, 20);
    let entry = folder_entry(&folder);
    let config = config(dir.path());

    let refusing = RecordingSink::refusing(2);
    let failure = run(&config, &entry, &refusing)
        .await
        .expect_err("the second batch is refused");
    assert!(
        matches!(failure.error, MemoryError::Unavailable(_)),
        "{failure:?}"
    );
    assert_eq!(failure.done.records_ingested, BATCH as u32);
    assert_eq!(refusing.items().len(), BATCH);

    let retry = RecordingSink::default();
    let outcome = run(&config, &entry, &retry).await.expect("retry");
    assert_eq!(retry.items().len(), 30 - BATCH, "only the refused batch");
    assert_eq!(outcome.records_ingested, (30 - BATCH) as u32);
}

/// Deleting a source's memory clears the record, so the next sync sends
/// everything again rather than trusting what was forgotten.
#[tokio::test]
async fn a_cleared_record_means_a_full_pass() {
    let dir = tempfile::tempdir().expect("tempdir");
    let folder = folder_of(dir.path(), &["a", "b"], 20);
    let entry = folder_entry(&folder);
    let config = config(dir.path());
    run(&config, &entry, &RecordingSink::default())
        .await
        .expect("first");
    forget_state(&config, &entry.id);
    let again = RecordingSink::default();
    run(&config, &entry, &again).await.expect("after clearing");
    assert_eq!(again.item_ids(), ["a.md", "b.md"]);
    // Clearing a record that is not there is a no-op.
    forget_state(&config, "never-synced");
}

/// A record that cannot be read is a full pass, never a failure.
#[tokio::test]
async fn an_unreadable_record_is_a_full_pass() {
    let dir = tempfile::tempdir().expect("tempdir");
    let folder = folder_of(dir.path(), &["a"], 20);
    let entry = folder_entry(&folder);
    let config = config(dir.path());
    let path = state_path(&config, &entry.id);
    std::fs::create_dir_all(path.parent().expect("parent")).expect("dir");
    std::fs::write(&path, b"not json").expect("garbage");
    let sink = RecordingSink::default();
    run(&config, &entry, &sink).await.expect("run");
    assert_eq!(sink.item_ids(), ["a.md"]);
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
    .await
    .expect_err("a missing folder");
    assert!(
        matches!(missing.error, MemoryError::Backend(_)),
        "{missing:?}"
    );
    assert_eq!(missing.done, SyncRunOutcome::default());
    let mut chats = folder_entry(dir.path());
    chats.kind = SourceKind::Conversation;
    let refused = run(&config(dir.path()), &chats, &sink)
        .await
        .expect_err("an unserved kind");
    assert!(
        matches!(refused.error, MemoryError::Unsupported { .. }),
        "{refused:?}"
    );
    assert!(sink.items().is_empty());
}
