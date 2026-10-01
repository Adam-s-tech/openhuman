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
//! It lists the source's items with the source's reader, keeps the newest
//! `max_items` and none older than `sync_depth_days`, reads each one, and hands
//! them to `accept_source_items` in batches. The sink skips an item it already
//! holds unchanged, so a run over an unchanged folder writes nothing, and a
//! changed item replaces its old version. `max_tokens_per_sync` stops the run
//! once the text read would pass it, at about four characters a token, and the
//! run reports more pending.
//!
//! # What it does not do
//!
//! An item removed from the source stays in memory: the sink forgets a whole
//! source, never one item by its id. Removing the source forgets everything it
//! synced.

use tinymemory_sources::types::ContentType;

use crate::config::Config;
use crate::memory::api::error::MemoryError;
use crate::memory::api::provider::sync::SyncRunOutcome;
use crate::memory::api::provider::types::SourceItem as SinkItem;
use crate::memory::api::provider::{MemoryProvider, MemorySourceSink};
use crate::memory::api::types::MemoryTaint;
use crate::memory::sources::readers::{folder, github, rss, web_page, SourceReader};
use crate::memory::sources::types::{MemorySourceEntry, SourceItem, SourceKind};

/// Items handed to the sink in one call.
const BATCH: usize = 25;

/// Characters a token is counted as, for `max_tokens_per_sync`.
const CHARS_PER_TOKEN: u64 = 4;

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

/// The items a run reads: none older than `sync_depth_days`, newest first, at
/// most `max_items`.
fn chosen(entry: &MemorySourceEntry, mut items: Vec<SourceItem>, now_ms: i64) -> Vec<SourceItem> {
    if let Some(days) = entry.sync_depth_days.filter(|days| *days > 0) {
        let floor = now_ms - i64::from(days) * 86_400_000;
        items.retain(|item| item.updated_at_ms.is_none_or(|at| at >= floor));
    }
    items.sort_by_key(|item| std::cmp::Reverse(item.updated_at_ms));
    if let Some(cap) = entry.max_items {
        items.truncate(cap as usize);
    }
    items
}

/// Syncs `entry` into `sink` once.
///
/// # Errors
///
/// [`MemoryError::Unsupported`] for a kind the host does not sync;
/// [`MemoryError::Backend`] when the source cannot be listed; the sink's own
/// error when a batch is refused. An item that cannot be read is skipped and
/// counted in the outcome's note.
pub(crate) async fn run(
    config: &Config,
    entry: &MemorySourceEntry,
    sink: &dyn MemorySourceSink,
) -> Result<SyncRunOutcome, MemoryError> {
    let Some(reader) = reader(&entry.kind) else {
        return Err(MemoryError::unsupported_raw(format!(
            "host sync of {} sources",
            entry.kind.as_str()
        )));
    };
    let listed = reader
        .list_items(entry, config)
        .await
        .map_err(|error| MemoryError::Backend(format!("listing source '{}': {error}", entry.id)))?;
    let total = listed.len();
    let items = chosen(entry, listed, chrono::Utc::now().timestamp_millis());
    tracing::debug!(
        source_id = %entry.id,
        kind = entry.kind.as_str(),
        listed = total,
        chosen = items.len(),
        "[memory_sources:hosted_sync] run starting"
    );
    let budget = entry
        .max_tokens_per_sync
        .map(|tokens| tokens.saturating_mul(CHARS_PER_TOKEN));
    let mut spent: u64 = 0;
    let mut unreadable = 0usize;
    let mut more_pending = false;
    let mut batch: Vec<SinkItem> = Vec::with_capacity(BATCH);
    let mut outcome = SyncRunOutcome::default();
    for item in items {
        let content = match reader.read_item(entry, &item.id, config).await {
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
        if content.body.trim().is_empty() {
            continue;
        }
        let size = content.body.len() as u64;
        if budget.is_some_and(|budget| spent + size > budget) {
            more_pending = true;
            break;
        }
        spent += size;
        batch.push(SinkItem {
            item_id: item.id,
            title: if content.title.is_empty() {
                item.title
            } else {
                content.title
            },
            content: content.body,
            mime: Some(mime_of(&content.content_type).to_string()),
            url: content
                .metadata
                .get("url")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string),
            updated_at_ms: item.updated_at_ms,
            tags: Vec::new(),
        });
        if batch.len() == BATCH {
            outcome.records_ingested += send(entry, sink, std::mem::take(&mut batch)).await?;
        }
    }
    if !batch.is_empty() {
        outcome.records_ingested += send(entry, sink, batch).await?;
    }
    outcome.more_pending = more_pending;
    if unreadable > 0 {
        outcome.note = Some(format!("{unreadable} item(s) could not be read"));
    }
    tracing::debug!(
        source_id = %entry.id,
        written = outcome.records_ingested,
        unreadable,
        more_pending,
        "[memory_sources:hosted_sync] run finished"
    );
    Ok(outcome)
}

/// One batch into the sink; answers how many items it wrote.
async fn send(
    entry: &MemorySourceEntry,
    sink: &dyn MemorySourceSink,
    batch: Vec<SinkItem>,
) -> Result<u32, MemoryError> {
    let accepted = sink
        .accept_source_items(&entry.id, entry.kind.as_str(), batch, taint_of(&entry.kind))
        .await?;
    Ok(accepted.written)
}

#[cfg(test)]
#[path = "hosted_sync_tests.rs"]
mod tests;
