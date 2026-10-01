//! Local sources the host syncs into a driver's source sink.
//!
//! The embedded engine runs its own pipeline behind `MemorySourceSync`: it
//! reads a folder, a repository, a feed or a page, tracks what changed, and
//! ingests it. Hosted memory runs none — it keeps what it is sent through its
//! source sink — so for a driver that serves the sink but not the pipeline
//! ([`host_synced`]), the host reads the source itself and sends the items.
//! The Sync button, Apply all and the periodic loop all come here for such a
//! driver.
//!
//! # What a run does
//!
//! It lists the source's items with the source's reader, newest first and none
//! older than `sync_depth_days`, and works only on what is new or changed since
//! the sink last accepted it: an item whose modified time matches what was
//! recorded is not read at all, and one read whose text is unchanged is not
//! sent. New and changed items go to `accept_source_items` in batches, and what
//! each accepted batch held is recorded once the sink has taken it.
//!
//! `max_items` caps the new or changed items a run sends, and
//! `max_tokens_per_sync` caps their text, at about four characters a token.
//! Either cap stops the run with more pending, and because what was sent is
//! recorded, the next run carries on past it rather than re-sending the same
//! newest items.
//!
//! The record is the host's, one small file per source, and it only ever saves
//! work: a missing or unreadable one is a full pass, which the sink makes
//! cheap by skipping what it already holds unchanged.
//!
//! # Removed items
//!
//! A file deleted from a synced folder is forgotten on the folder's next sync:
//! the record keeps the id the sink answered for each item, and an item the
//! walk no longer finds is forgotten by that id (`ForgetSelector::Chunk`). Only
//! a folder's listing is the whole source; an item missing from a feed or a
//! repository listing may simply be past its window, so those stay. An empty
//! listing is never read as "everything was deleted".
//!
//! Removing a source from the registry stops its syncs and keeps what it
//! synced, as on the local engine; deleting the source's memory
//! (`memory_tree.delete_source`) forgets all of it and clears this record, so a
//! later sync starts over.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock, PoisonError};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tinymemory_sources::types::ContentType;

use crate::config::Config;
use crate::memory::api::error::MemoryError;
use crate::memory::api::provider::sync::SyncRunOutcome;
use crate::memory::api::provider::types::SourceItem as SinkItem;
use crate::memory::api::provider::types::{ForgetSelector, IngestOutcome};
use crate::memory::api::provider::{MemoryProvider, MemorySourceSink};
use crate::memory::api::types::MemoryTaint;
use crate::memory::sources::readers::{folder, github, rss, web_page, SourceReader};
use crate::memory::sources::rpc::driver_run::PartialFailure;
use crate::memory::sources::types::{MemorySourceEntry, SourceContent, SourceItem, SourceKind};

/// Items handed to the sink in one call.
const BATCH: usize = 25;

/// Characters a token is counted as, for `max_tokens_per_sync`.
const CHARS_PER_TOKEN: u64 = 4;

/// Where the per-source records live, under the workspace's state directory.
const STATE_DIR: &str = "state/hosted_sync";

/// Whether the host syncs local sources for `provider`: it keeps what it is
/// sent, and runs no source pipeline of its own.
pub(crate) fn host_synced(provider: &dyn MemoryProvider) -> bool {
    provider.as_source_sync().is_none() && provider.as_sources().is_some()
}

/// Whether the host syncs sources of `kind`. Connector sources go through the
/// connector, and conversations and X queries have no host reader that feeds a
/// sink.
pub(crate) fn serves(kind: &SourceKind) -> bool {
    matches!(
        kind,
        SourceKind::Folder | SourceKind::GithubRepo | SourceKind::RssFeed | SourceKind::WebPage
    )
}

/// The reader for `kind`, built here rather than taken from `reader_for`: a
/// run may come from the periodic loop as well as a user's request, and the
/// network readers are handed out only for the kinds a user added as sources
/// to be synced on their schedule.
fn reader(kind: &SourceKind) -> Option<Box<dyn SourceReader>> {
    match kind {
        SourceKind::Folder => Some(Box::new(folder::FolderReader)),
        SourceKind::GithubRepo => Some(Box::new(github::GithubReader)),
        SourceKind::RssFeed => Some(Box::new(rss::RssReader::new())),
        SourceKind::WebPage => Some(Box::new(web_page::WebPageReader)),
        _ => None,
    }
}

/// A local folder is the user's own material; anything fetched is not.
fn taint_of(kind: &SourceKind) -> MemoryTaint {
    if *kind == SourceKind::Folder {
        MemoryTaint::Internal
    } else {
        MemoryTaint::ExternalSync
    }
}

fn mime_of(content_type: &ContentType) -> &'static str {
    match content_type {
        ContentType::Markdown => "text/markdown",
        ContentType::Html => "text/html",
        ContentType::Plaintext => "text/plain",
    }
}

/// The items a run considers: none older than `sync_depth_days`, newest first.
fn chosen(entry: &MemorySourceEntry, mut items: Vec<SourceItem>, now_ms: i64) -> Vec<SourceItem> {
    if let Some(days) = entry.sync_depth_days.filter(|days| *days > 0) {
        let floor = now_ms - i64::from(days) * 86_400_000;
        items.retain(|item| item.updated_at_ms.is_none_or(|at| at >= floor));
    }
    items.sort_by_key(|item| std::cmp::Reverse(item.updated_at_ms));
    items
}

/// One item as the sink last took it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Seen {
    /// The item's modified time then, when its reader gives one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    at: Option<i64>,
    /// A digest of what was sent: its title, text and URL.
    digest: String,
    /// The id the sink answered for the item, when its answer lined up with
    /// the batch: what forgets the item once its source no longer lists it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    stored_as: Option<String>,
}

/// Whether a source's listing is the whole source, so an item missing from it
/// was removed rather than merely not fetched. A folder walk lists every file;
/// a feed or a repository listing is a window onto something larger.
fn removes_vanished(kind: &SourceKind) -> bool {
    *kind == SourceKind::Folder
}

/// What a source's earlier runs handed the sink, by item id.
#[derive(Debug, Default, Serialize, Deserialize)]
struct SyncState {
    #[serde(default)]
    items: HashMap<String, Seen>,
}

/// The file holding `source_id`'s record. The name is a digest, so any source
/// id makes a safe file name.
fn state_path(config: &Config, source_id: &str) -> PathBuf {
    let digest = Sha256::digest(source_id.as_bytes());
    let name: String = digest[..8].iter().map(|b| format!("{b:02x}")).collect();
    config
        .workspace_dir
        .join(STATE_DIR)
        .join(format!("{name}.json"))
}

/// The record at `path`, or an empty one when there is none or it cannot be
/// read: a full pass costs lookups, never wrong data.
fn load_state(path: &Path) -> SyncState {
    match std::fs::read(path) {
        Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_else(|error| {
            tracing::warn!(
                path = %path.display(),
                "[memory_sources:hosted_sync] sync record unreadable; doing a full pass: {error}"
            );
            SyncState::default()
        }),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => SyncState::default(),
        Err(error) => {
            tracing::warn!(
                path = %path.display(),
                "[memory_sources:hosted_sync] sync record unreadable; doing a full pass: {error}"
            );
            SyncState::default()
        }
    }
}

/// Saves the record, replacing the old one whole. A failure is logged and not
/// fatal: the next run re-sends what this one could not record, and the sink
/// skips it.
fn save_state(path: &Path, state: &SyncState) {
    let write = || -> std::io::Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, serde_json::to_vec(state)?)?;
        std::fs::rename(&tmp, path)
    };
    if let Err(error) = write() {
        tracing::warn!(
            path = %path.display(),
            "[memory_sources:hosted_sync] could not save the sync record: {error}"
        );
    }
}

/// Forgets what was recorded for `source_id`, so its next sync is a full pass.
/// Called when the source's memory is deleted or the source is removed.
pub(crate) fn forget_state(config: &Config, source_id: &str) {
    let path = state_path(config, source_id);
    match std::fs::remove_file(&path) {
        Ok(()) => tracing::debug!(
            path = %path.display(),
            "[memory_sources:hosted_sync] sync record cleared"
        ),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => tracing::warn!(
            path = %path.display(),
            "[memory_sources:hosted_sync] could not clear the sync record: {error}"
        ),
    }
}

/// The lock one source's runs take, so a manual sync and a scheduled one do
/// not read and send the same items at once.
fn source_lock(source_id: &str) -> Arc<tokio::sync::Mutex<()>> {
    static LOCKS: OnceLock<Mutex<HashMap<String, Arc<tokio::sync::Mutex<()>>>>> = OnceLock::new();
    LOCKS
        .get_or_init(Default::default)
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .entry(source_id.to_string())
        .or_default()
        .clone()
}

fn digest_of(title: &str, content: &SourceContent, url: Option<&str>) -> String {
    let mut hasher = Sha256::new();
    for part in [title, &content.body, url.unwrap_or_default()] {
        hasher.update((part.len() as u64).to_le_bytes());
        hasher.update(part.as_bytes());
    }
    hasher.finalize()[..16]
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// A run's batch in flight, and what it has done so far.
struct Progress {
    path: PathBuf,
    state: SyncState,
    batch: Vec<SinkItem>,
    seen: Vec<(String, Seen)>,
    outcome: SyncRunOutcome,
}

impl Progress {
    /// Sends the batch and, once the sink has taken it, records what it held.
    async fn flush(
        &mut self,
        entry: &MemorySourceEntry,
        sink: &dyn MemorySourceSink,
    ) -> Result<(), MemoryError> {
        if self.batch.is_empty() {
            return Ok(());
        }
        let accepted = send(entry, sink, std::mem::take(&mut self.batch)).await?;
        self.outcome.records_ingested += accepted.written;
        // The sink answers one id per item, in order, when it can; a short or
        // long answer cannot be matched to items and is not guessed at.
        let aligned = accepted.ids.len() == self.seen.len();
        for (index, (id, mut seen)) in self.seen.drain(..).enumerate() {
            if aligned {
                seen.stored_as = accepted.ids.get(index).cloned();
            }
            self.state.items.insert(id, seen);
        }
        save_state(&self.path, &self.state);
        Ok(())
    }
}

/// Syncs `entry` into `sink` once.
///
/// # Errors
///
/// [`MemoryError::Unsupported`] for a kind the host does not sync;
/// [`MemoryError::Backend`] when the source cannot be listed; the sink's own
/// error when a batch is refused, with what earlier batches wrote. An item
/// that cannot be read is skipped and counted in the outcome's note.
pub(in crate::memory::sources) async fn run(
    config: &Config,
    entry: &MemorySourceEntry,
    sink: &dyn MemorySourceSink,
) -> Result<SyncRunOutcome, PartialFailure> {
    let Some(reader) = reader(&entry.kind) else {
        return Err(MemoryError::unsupported_raw(format!(
            "host sync of {} sources",
            entry.kind.as_str()
        ))
        .into());
    };
    let lock = source_lock(&entry.id);
    let _running = lock.lock().await;
    let listed = reader
        .list_items(entry, &config.workspace_dir)
        .await
        .map_err(|error| MemoryError::Backend(format!("listing source '{}': {error}", entry.id)))?;
    let listed_ids: HashSet<String> = listed.iter().map(|item| item.id.clone()).collect();
    let total = listed.len();
    let items = chosen(entry, listed, chrono::Utc::now().timestamp_millis());
    let path = state_path(config, &entry.id);
    let mut state = load_state(&path);
    // An item the source no longer lists has nothing left to compare. When
    // the listing is the whole source, it was removed, and its memory goes
    // too. An empty listing is not taken as "every file was deleted": that is
    // far likelier a folder that could not be walked.
    let vanished: Vec<(String, Seen)> = if removes_vanished(&entry.kind) && !listed_ids.is_empty() {
        state
            .items
            .iter()
            .filter(|(id, _)| !listed_ids.contains(*id))
            .map(|(id, seen)| (id.clone(), seen.clone()))
            .collect()
    } else {
        Vec::new()
    };
    state.items.retain(|id, _| listed_ids.contains(id));
    let forgotten = forget_vanished(entry, sink, vanished, &mut state).await;
    tracing::debug!(
        source_id = %entry.id,
        kind = entry.kind.as_str(),
        listed = total,
        considered = items.len(),
        recorded = state.items.len(),
        "[memory_sources:hosted_sync] run starting"
    );
    let budget = entry
        .max_tokens_per_sync
        .map(|tokens| tokens.saturating_mul(CHARS_PER_TOKEN));
    let cap = entry.max_items.map(|cap| cap as usize);
    let mut spent: u64 = 0;
    let mut queued = 0usize;
    let mut unreadable = 0usize;
    let mut unchanged = 0usize;
    let mut more_pending = false;
    let mut progress = Progress {
        path,
        state,
        batch: Vec::with_capacity(BATCH),
        seen: Vec::with_capacity(BATCH),
        outcome: SyncRunOutcome::default(),
    };
    for item in items {
        let held = progress.state.items.get(&item.id);
        if item.updated_at_ms.is_some() && held.is_some_and(|held| held.at == item.updated_at_ms) {
            unchanged += 1;
            continue;
        }
        if cap.is_some_and(|cap| queued >= cap) {
            more_pending = true;
            break;
        }
        let content = match reader
            .read_item(entry, &item.id, &config.workspace_dir)
            .await
        {
            Ok(content) => content,
            Err(error) => {
                unreadable += 1;
                tracing::warn!(
                    source_id = %entry.id,
                    item_id = %item.id,
                    "[memory_sources:hosted_sync] item could not be read: {error}"
                );
                continue;
            }
        };
        let title = if content.title.is_empty() {
            item.title.clone()
        } else {
            content.title.clone()
        };
        let url = content
            .metadata
            .get("url")
            .and_then(serde_json::Value::as_str)
            .map(str::to_string);
        let seen = Seen {
            at: item.updated_at_ms,
            digest: digest_of(&title, &content, url.as_deref()),
            // Kept until the sink answers for the new text, so an item that is
            // not sent again can still be forgotten.
            stored_as: progress
                .state
                .items
                .get(&item.id)
                .and_then(|held| held.stored_as.clone()),
        };
        let same_text = progress
            .state
            .items
            .get(&item.id)
            .is_some_and(|held| held.digest == seen.digest);
        if same_text || content.body.trim().is_empty() {
            // Nothing to send. Recording it now lets the next run skip it
            // without reading it, by its modified time.
            unchanged += 1;
            progress.state.items.insert(item.id, seen);
            continue;
        }
        let size = content.body.len() as u64;
        if budget.is_some_and(|budget| spent + size > budget) {
            more_pending = true;
            break;
        }
        spent += size;
        queued += 1;
        progress.batch.push(SinkItem {
            item_id: item.id.clone(),
            title,
            content: content.body,
            mime: Some(mime_of(&content.content_type).to_string()),
            url,
            updated_at_ms: item.updated_at_ms,
            tags: Vec::new(),
        });
        progress.seen.push((item.id, seen));
        if progress.batch.len() == BATCH {
            if let Err(error) = progress.flush(entry, sink).await {
                return Err(failed(progress, error, more_pending));
            }
        }
    }
    if let Err(error) = progress.flush(entry, sink).await {
        return Err(failed(progress, error, more_pending));
    }
    // What was read and found unchanged is recorded even when nothing was sent.
    save_state(&progress.path, &progress.state);
    let mut outcome = progress.outcome;
    outcome.more_pending = more_pending;
    let mut notes = Vec::new();
    if unreadable > 0 {
        notes.push(format!("{unreadable} item(s) could not be read"));
    }
    if forgotten > 0 {
        notes.push(format!("{forgotten} removed item(s) forgotten"));
    }
    if !notes.is_empty() {
        outcome.note = Some(notes.join("; "));
    }
    tracing::debug!(
        source_id = %entry.id,
        written = outcome.records_ingested,
        unchanged,
        unreadable,
        forgotten,
        more_pending,
        "[memory_sources:hosted_sync] run finished"
    );
    Ok(outcome)
}

/// A refused batch, with what the run had done before it. The batch that was
/// refused is not recorded, so the next run sends it again.
fn failed(progress: Progress, error: MemoryError, more_pending: bool) -> PartialFailure {
    save_state(&progress.path, &progress.state);
    let mut done = progress.outcome;
    done.more_pending = more_pending;
    PartialFailure { error, done }
}

/// One batch into the sink.
async fn send(
    entry: &MemorySourceEntry,
    sink: &dyn MemorySourceSink,
    batch: Vec<SinkItem>,
) -> Result<IngestOutcome, MemoryError> {
    sink.accept_source_items(&entry.id, entry.kind.as_str(), batch, taint_of(&entry.kind))
        .await
}

/// Forgets each vanished item the sink answered an id for, and answers how
/// many the sink removed. One it could not forget goes back into `state`, so
/// the next run, finding it still unlisted, tries again; one with no id is
/// beyond reach and is only dropped from the record.
async fn forget_vanished(
    entry: &MemorySourceEntry,
    sink: &dyn MemorySourceSink,
    vanished: Vec<(String, Seen)>,
    state: &mut SyncState,
) -> u64 {
    let mut forgotten = 0u64;
    for (item_id, seen) in vanished {
        let Some(stored_as) = seen.stored_as.clone() else {
            tracing::debug!(
                source_id = %entry.id,
                item_id = %item_id,
                "[memory_sources:hosted_sync] removed item has no stored id to forget by"
            );
            continue;
        };
        let selector = ForgetSelector::Chunk {
            chunk_id: stored_as,
        };
        match sink.forget_matching(&selector).await {
            Ok(outcome) => {
                forgotten += outcome.chunks_removed;
                tracing::debug!(
                    source_id = %entry.id,
                    item_id = %item_id,
                    removed = outcome.chunks_removed,
                    "[memory_sources:hosted_sync] removed item forgotten"
                );
            }
            Err(error) => {
                tracing::warn!(
                    source_id = %entry.id,
                    item_id = %item_id,
                    "[memory_sources:hosted_sync] could not forget a removed item; will retry: {error}"
                );
                state.items.insert(item_id, seen);
            }
        }
    }
    forgotten
}

#[cfg(test)]
#[path = "hosted_sync_tests.rs"]
mod tests;
