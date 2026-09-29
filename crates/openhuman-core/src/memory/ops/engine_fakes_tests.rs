//! A scripted provider for the engine and fallback tests: it serves a fixed
//! entry list through the mandatory families, counts every call, pages its
//! export by the requested size, and answers `import_records` with a scripted
//! outcome. Optionally hangs in `export_page` (for cancel/timeout cases).

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;

use crate::memory::api::capabilities::Capabilities;
use crate::memory::api::error::MemoryError;
use crate::memory::api::health::MemoryHealth;
use crate::memory::api::provider::types::{
    ExportPage, ExportRecord, ImportOutcome, SourceScope,
};
use crate::memory::api::provider::{MemoryCore, MemoryPortability, MemoryProvider, MemoryRecall};
use crate::memory::api::recall::OwnedRecallOpts;
use crate::memory::api::types::{MemoryCategory, MemoryEntry, MemoryTaint, NamespaceSummary};

#[derive(Default)]
pub(crate) struct ScriptedProvider {
    entries: Vec<MemoryEntry>,
    pub list_calls: AtomicUsize,
    pub namespaces_calls: AtomicUsize,
    pub export_calls: AtomicUsize,
    pub import_calls: AtomicUsize,
    /// The largest `limit` any `export_page` call asked for.
    pub max_export_limit: AtomicUsize,
    pub imported: Mutex<Vec<ExportRecord>>,
    import_outcome: Mutex<ImportOutcome>,
    hang_on_export: bool,
    panic_on_export: bool,
}

pub(crate) fn entry(namespace: &str, key: &str, content: &str, ts: &str) -> MemoryEntry {
    MemoryEntry {
        id: format!("{namespace}/{key}"),
        key: key.to_string(),
        content: content.to_string(),
        namespace: Some(namespace.to_string()),
        category: MemoryCategory::Core,
        timestamp: ts.to_string(),
        session_id: None,
        score: None,
        taint: MemoryTaint::Internal,
    }
}

impl ScriptedProvider {
    pub(crate) fn new(entries: Vec<MemoryEntry>) -> Arc<Self> {
        Arc::new(Self {
            entries,
            ..Self::default()
        })
    }

    /// `n` entries spread over `namespaces` namespaces (`ns0`, `ns1`, ...).
    pub(crate) fn spread(n: usize, namespaces: usize) -> Arc<Self> {
        Self::new(
            (0..n)
                .map(|i| {
                    entry(
                        &format!("ns{}", i % namespaces),
                        &format!("k{i}"),
                        &format!("content {i}"),
                        &format!("2026-01-01T00:{:02}:{:02}Z", (i / 60) % 60, i % 60),
                    )
                })
                .collect(),
        )
    }

    pub(crate) fn with_import(self: Arc<Self>, outcome: ImportOutcome) -> Arc<Self> {
        *self.import_outcome.lock().unwrap() = outcome;
        self
    }

    pub(crate) fn hanging(self: Arc<Self>) -> Arc<Self> {
        let mut inner = Arc::try_unwrap(self).ok().expect("unshared");
        inner.hang_on_export = true;
        Arc::new(inner)
    }

    pub(crate) fn panicking(self: Arc<Self>) -> Arc<Self> {
        let mut inner = Arc::try_unwrap(self).ok().expect("unshared");
        inner.panic_on_export = true;
        Arc::new(inner)
    }
}

#[async_trait]
impl MemoryCore for ScriptedProvider {
    async fn store(
        &self,
        _namespace: &str,
        _key: &str,
        _content: &str,
        _category: MemoryCategory,
        _session_id: Option<&str>,
        _taint: MemoryTaint,
    ) -> Result<(), MemoryError> {
        Ok(())
    }

    async fn get(&self, _namespace: &str, _key: &str) -> Result<Option<MemoryEntry>, MemoryError> {
        Ok(None)
    }

    async fn forget(&self, _namespace: &str, _key: &str) -> Result<bool, MemoryError> {
        Ok(false)
    }

    async fn list(
        &self,
        namespace: Option<&str>,
        _category: Option<&MemoryCategory>,
        _session_id: Option<&str>,
    ) -> Result<Vec<MemoryEntry>, MemoryError> {
        self.list_calls.fetch_add(1, Ordering::SeqCst);
        Ok(self
            .entries
            .iter()
            .filter(|e| namespace.is_none_or(|ns| e.namespace.as_deref() == Some(ns)))
            .cloned()
            .collect())
    }

    async fn namespaces(&self) -> Result<Vec<NamespaceSummary>, MemoryError> {
        self.namespaces_calls.fetch_add(1, Ordering::SeqCst);
        let mut names: Vec<String> = self
            .entries
            .iter()
            .filter_map(|e| e.namespace.clone())
            .collect();
        names.sort();
        names.dedup();
        Ok(names
            .into_iter()
            .map(|namespace| NamespaceSummary {
                namespace,
                count: 0,
                last_updated: None,
            })
            .collect())
    }
}

#[async_trait]
impl MemoryRecall for ScriptedProvider {
    async fn recall(
        &self,
        _query: &str,
        _limit: usize,
        _opts: &OwnedRecallOpts,
        _scope: Option<&SourceScope>,
    ) -> Result<Vec<MemoryEntry>, MemoryError> {
        Ok(Vec::new())
    }
}

#[async_trait]
impl MemoryPortability for ScriptedProvider {
    async fn export_page(
        &self,
        cursor: Option<&str>,
        limit: usize,
    ) -> Result<ExportPage, MemoryError> {
        self.export_calls.fetch_add(1, Ordering::SeqCst);
        self.max_export_limit.fetch_max(limit, Ordering::SeqCst);
        if self.panic_on_export {
            panic!("scripted export panic");
        }
        if self.hang_on_export {
            std::future::pending::<()>().await;
        }
        let start: usize = cursor.and_then(|c| c.parse().ok()).unwrap_or(0);
        let end = (start + limit).min(self.entries.len());
        let records = self.entries[start..end]
            .iter()
            .cloned()
            .map(tinymemory_api::mandatory::to_record)
            .collect();
        Ok(ExportPage {
            records,
            next_cursor: (end < self.entries.len()).then(|| end.to_string()),
        })
    }

    async fn import_records(
        &self,
        records: Vec<ExportRecord>,
    ) -> Result<ImportOutcome, MemoryError> {
        self.import_calls.fetch_add(1, Ordering::SeqCst);
        let mut outcome = self.import_outcome.lock().unwrap().clone();
        if outcome == ImportOutcome::default() {
            outcome.imported = records.len() as u32;
        }
        self.imported.lock().unwrap().extend(records);
        Ok(outcome)
    }
}

#[async_trait]
impl MemoryProvider for ScriptedProvider {
    fn driver_id(&self) -> &str {
        "scripted"
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities::mandatory()
    }

    async fn health(&self) -> MemoryHealth {
        MemoryHealth::Ready
    }
}
