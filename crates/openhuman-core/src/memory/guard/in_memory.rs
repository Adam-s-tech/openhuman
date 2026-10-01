//! Guard-wrapping helpers over the conformance crate's in-memory drivers.
//!
//! # Why the drivers are not here
//!
//! [`InMemoryProvider`] (a substring-recall store that actually retains, for
//! round-trip assertions) and [`FixedRecallProvider`] (a constant `recall`, for
//! tests about what a caller does with a scripted result set) are engine-neutral
//! reference drivers, and live in `tinymemory-conformance`. What is host-specific
//! is wrapping one in a real [`MemoryGuard`](super::MemoryGuard) at the trusted
//! tier, so a consumer converted to hold a guard can be tested through the same
//! policy decorator production uses; that is all this module keeps.
//!
//! `RecordingProvider` (`test_support`) cannot stand in for either: it records
//! calls, which proves a call was *made* but never that data came back.

use std::sync::Arc;

use crate::memory::api::provider::MemoryProvider;
use crate::memory::api::types::MemoryEntry;

pub use tinymemory_conformance::{FixedRecallProvider, InMemoryProvider};

/// A real [`MemoryGuard`](super::MemoryGuard) over a fresh in-memory store,
/// plus the store itself for direct assertions.
#[must_use]
pub fn guarded_in_memory() -> (Arc<InMemoryProvider>, Arc<super::MemoryGuard>) {
    let provider = Arc::new(InMemoryProvider::new());
    let guard = guard_over(Arc::clone(&provider) as Arc<dyn MemoryProvider>);
    (provider, guard)
}

/// Wrap any provider in a real [`MemoryGuard`](super::MemoryGuard) at the
/// trusted tier.
///
/// Split out of [`guarded_in_memory`] so a test that needs an optional family
/// — retrieval, say — can supply its own provider and still be exercised
/// through the same policy decorator production uses, rather than calling the
/// provider directly and skipping the guard entirely.
#[must_use]
pub fn guard_over(provider: Arc<dyn MemoryProvider>) -> Arc<super::MemoryGuard> {
    let policy = Arc::new(super::HostGuardPolicy::new(
        "in-memory",
        crate::core::subsystem::DriverClass::Embedded,
        crate::config::schema::MemoryHooksConfig::default(),
        super::policy::TRUSTED,
    ));
    Arc::new(super::MemoryGuard::new(provider, policy))
}

/// A real [`MemoryGuard`](super::MemoryGuard) over a [`FixedRecallProvider`]
/// that answers `entries` to every recall, ready to drop into a context.
#[must_use]
pub fn guarded_fixed_recall(entries: Vec<MemoryEntry>) -> Arc<super::MemoryGuard> {
    guard_over(Arc::new(FixedRecallProvider::new(entries)) as Arc<dyn MemoryProvider>)
}
