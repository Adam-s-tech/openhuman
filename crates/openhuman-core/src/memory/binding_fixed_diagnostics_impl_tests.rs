use crate::memory::api::error::MemoryError;
use crate::memory::api::provider::types::{
    ExportPage, ExportRecord, ImportOutcome, MaintenanceReport, QueueFailure, QueueStats,
    SourceScope, StoreStats,
};
use crate::memory::api::provider::{
    MemoryCore, MemoryMaintenance, MemoryPortability, MemoryProvider, MemoryRecall,
};
use crate::memory::api::recall::OwnedRecallOpts;
use crate::memory::api::types::{MemoryCategory, MemoryEntry, MemoryTaint, NamespaceSummary};
use async_trait::async_trait;
use tinymemory_api::null::NullMemoryProvider;

/// A driver that reports the diagnostics it was handed, and does nothing else.
///
/// Reads that used to hit the engine's tables go through the contract now, and
/// the real driver is a compiled module that cannot load inside a unit test —
/// so a test workspace binds the null driver and every diagnostic answers
/// empty. A handler that used to be provable by writing rows and calling it
/// needs a driver in between.
///
/// The split that leaves is the honest one. What a handler *derives* from the
/// numbers is the host's rule and belongs in the host's tests, which is what
/// this exists for. What a given store *is* — that an ingest raises the chunk
/// count, that a deferred job stays ready without becoming eligible — is the
/// driver's rule, pinned in the driver's own conformance suite against a real
/// store.
///
/// Everything outside `Maintenance` delegates to the null driver: a test that
/// needed those would be testing something this double is the wrong shape for.
pub(crate) struct FixedDiagnostics {
    inner: NullMemoryProvider,
    /// How many times the host has asked this driver to retry failed work,
    /// and how many jobs it should say it requeued when asked.
    ///
    /// The gate in front of the ask is host logic — only an embedder change
    /// should un-park anything — so a test needs to see whether the ask
    /// happened, separately from what the driver would have done.
    retry_calls: std::sync::atomic::AtomicUsize,
    retry_requeues: u64,
    /// How many times the host has asked this driver to re-embed.
    ///
    /// `reembed` enqueues work rather than doing it, so the host's side of that
    /// contract is only that it *asked* — whether a row appears is the driver's
    /// business, and pinning it here would test the driver through the host.
    reembed_calls: std::sync::atomic::AtomicUsize,
    store: crate::memory::api::provider::types::StoreStats,
    queue: crate::memory::api::provider::types::QueueStats,
    failure: Option<crate::memory::api::provider::types::QueueFailure>,
    /// What this driver says about a backfill running in its process.
    ///
    /// Separate from [`Self::queue`] on purpose, mirroring the contract: the
    /// flag is not derivable from the counts, and a test that needs the gap
    /// between them — nothing ready, nothing running, backfill unfinished —
    /// has to set the two independently.
    backfill: bool,
    /// What [`MemoryMaintenance::backfill_connector_trees`] answers, when a test
    /// sets it.
    ///
    /// Distinct from [`Self::backfill`], which is the unrelated
    /// `backfill_in_progress` flag — one is "is a re-embed running", the other
    /// is the connector-tree pass's counters.
    backfill_trees: crate::memory::api::provider::types::BackfillTreesOutcome,
    /// What [`MemoryMaintenance::flush_pending`] answers, when a test sets it.
    flush: crate::memory::api::provider::types::FlushOutcome,
    /// What [`MemoryMaintenance::reset_derived_index`] answers, likewise.
    reset: crate::memory::api::provider::types::ResetOutcome,
}

impl FixedDiagnostics {
    pub(crate) fn new(store: StoreStats, queue: QueueStats) -> Self {
        Self {
            inner: NullMemoryProvider::new(),
            store,
            queue,
            failure: None,
            backfill: false,
            backfill_trees: Default::default(),
            flush: Default::default(),
            reset: Default::default(),
            retry_calls: std::sync::atomic::AtomicUsize::new(0),
            retry_requeues: 0,
            reembed_calls: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    /// Report `requeued` jobs from [`MemoryMaintenance::retry_failed`].
    pub(crate) fn requeueing(mut self, requeued: u64) -> Self {
        self.retry_requeues = requeued;
        self
    }

    /// Report a backfill running in this driver's process.
    pub(crate) fn backfilling(mut self) -> Self {
        self.backfill = true;
        self
    }

    /// Answer [`MemoryMaintenance::backfill_connector_trees`] with `outcome`.
    ///
    /// Named apart from [`Self::backfilling`], which sets the unrelated
    /// `backfill_in_progress` flag.
    pub(crate) fn backfilling_trees(
        mut self,
        outcome: crate::memory::api::provider::types::BackfillTreesOutcome,
    ) -> Self {
        self.backfill_trees = outcome;
        self
    }

    /// Answer [`MemoryMaintenance::flush_pending`] with `outcome`.
    pub(crate) fn flushing(
        mut self,
        outcome: crate::memory::api::provider::types::FlushOutcome,
    ) -> Self {
        self.flush = outcome;
        self
    }

    /// Answer [`MemoryMaintenance::reset_derived_index`] with `outcome`.
    pub(crate) fn resetting(
        mut self,
        outcome: crate::memory::api::provider::types::ResetOutcome,
    ) -> Self {
        self.reset = outcome;
        self
    }

    /// How many times [`MemoryMaintenance::retry_failed`] has been called.
    pub(crate) fn retry_calls(&self) -> usize {
        self.retry_calls.load(std::sync::atomic::Ordering::SeqCst)
    }

    /// How many times [`MemoryMaintenance::reembed`] has been called.
    pub(crate) fn reembed_calls(&self) -> usize {
        self.reembed_calls.load(std::sync::atomic::Ordering::SeqCst)
    }
}

#[async_trait]
impl MemoryCore for FixedDiagnostics {
    async fn store(
        &self,
        namespace: &str,
        key: &str,
        content: &str,
        category: MemoryCategory,
        session_id: Option<&str>,
        taint: MemoryTaint,
    ) -> Result<(), MemoryError> {
        self.inner
            .store(namespace, key, content, category, session_id, taint)
            .await
    }

    async fn get(&self, namespace: &str, key: &str) -> Result<Option<MemoryEntry>, MemoryError> {
        self.inner.get(namespace, key).await
    }

    async fn forget(&self, namespace: &str, key: &str) -> Result<bool, MemoryError> {
        self.inner.forget(namespace, key).await
    }

    async fn list(
        &self,
        namespace: Option<&str>,
        category: Option<&MemoryCategory>,
        session_id: Option<&str>,
    ) -> Result<Vec<MemoryEntry>, MemoryError> {
        self.inner.list(namespace, category, session_id).await
    }

    async fn namespaces(&self) -> Result<Vec<NamespaceSummary>, MemoryError> {
        self.inner.namespaces().await
    }
}

#[async_trait]
impl MemoryRecall for FixedDiagnostics {
    async fn recall(
        &self,
        query: &str,
        limit: usize,
        opts: &OwnedRecallOpts,
        scope: Option<&SourceScope>,
    ) -> Result<Vec<MemoryEntry>, MemoryError> {
        self.inner.recall(query, limit, opts, scope).await
    }
}

#[async_trait]
impl MemoryPortability for FixedDiagnostics {
    async fn export_page(
        &self,
        cursor: Option<&str>,
        limit: usize,
    ) -> Result<ExportPage, MemoryError> {
        self.inner.export_page(cursor, limit).await
    }

    async fn import_records(
        &self,
        records: Vec<ExportRecord>,
    ) -> Result<ImportOutcome, MemoryError> {
        self.inner.import_records(records).await
    }
}

#[async_trait]
impl MemoryMaintenance for FixedDiagnostics {
    async fn retry_failed(&self) -> Result<MaintenanceReport, MemoryError> {
        self.retry_calls
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Ok(MaintenanceReport {
            operation: "retry_failed".to_string(),
            examined: self.retry_requeues,
            changed: self.retry_requeues,
            findings: Vec::new(),
        })
    }

    async fn reembed(&self) -> Result<MaintenanceReport, MemoryError> {
        self.reembed_calls
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Ok(MaintenanceReport::default())
    }

    async fn compact(&self) -> Result<MaintenanceReport, MemoryError> {
        Ok(MaintenanceReport::default())
    }

    async fn consolidate(&self) -> Result<MaintenanceReport, MemoryError> {
        Ok(MaintenanceReport::default())
    }

    async fn doctor(&self) -> Result<MaintenanceReport, MemoryError> {
        Ok(MaintenanceReport::default())
    }

    async fn store_stats(&self) -> Result<StoreStats, MemoryError> {
        Ok(self.store.clone())
    }

    async fn queue_stats(&self, _kind: Option<&str>) -> Result<QueueStats, MemoryError> {
        Ok(self.queue.clone())
    }

    async fn latest_queue_failure(&self) -> Result<Option<QueueFailure>, MemoryError> {
        Ok(self.failure.clone())
    }

    async fn backfill_in_progress(&self) -> Result<bool, MemoryError> {
        Ok(self.backfill)
    }

    async fn flush_pending(
        &self,
    ) -> Result<crate::memory::api::provider::types::FlushOutcome, MemoryError> {
        Ok(self.flush.clone())
    }

    async fn backfill_connector_trees(
        &self,
        _request: crate::memory::api::provider::types::BackfillTreesRequest,
    ) -> Result<crate::memory::api::provider::types::BackfillTreesOutcome, MemoryError> {
        Ok(self.backfill_trees.clone())
    }

    async fn reset_derived_index(
        &self,
    ) -> Result<crate::memory::api::provider::types::ResetOutcome, MemoryError> {
        Ok(self.reset.clone())
    }
}

#[async_trait]
impl MemoryProvider for FixedDiagnostics {
    fn driver_id(&self) -> &str {
        "fixed-diagnostics"
    }

    fn capabilities(&self) -> crate::memory::api::capabilities::Capabilities {
        crate::memory::api::capabilities::Capabilities::all()
    }

    async fn health(&self) -> crate::memory::api::health::MemoryHealth {
        crate::memory::api::health::MemoryHealth::Ready
    }

    fn as_maintenance(&self) -> Option<&dyn MemoryMaintenance> {
        Some(self)
    }
}
