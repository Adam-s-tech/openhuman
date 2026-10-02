//! Step 7, host half: the guard's log lines and its audit event.
//!
//! The span, the char-budget trace and `NO_NAMESPACE` live with the decorator in
//! `tinymemory_guard::audit`; what stays here is what needs the host — the
//! driver class in the log line and the bus.
//!
//! ## What may be logged, and what may never be
//!
//! Memory content is the most sensitive data in the product. Nothing here —
//! log line or bus event — carries a memory body, a recall query, a namespace
//! *key*, or a document title. What it carries is **shapes**: the driver id, the
//! contract method, the namespace, char counts, and hit counts.
//!
//! That is also why this module publishes a purpose-built
//! [`DomainEvent::MemoryGuardDenied`] rather than reusing
//! [`DomainEvent::MemoryRecalled`], which carries the raw query string. Firing
//! that one from the guard would push user query text onto the bus on every
//! call.
//!
//! ## Denials only on the bus
//!
//! Success is logged (at `debug`, into the file-only core log) but is **not**
//! published. One bus event per memory read would flood every subscriber on the
//! hot path for no operator benefit; a refusal is the rare, actionable event.

use crate::core::bus::BUS;
use crate::core::events::DomainEvent;
use tinymemory_guard::audit::LOG_PREFIX;
use tinymemory_guard::GuardPolicy;

use super::policy::HostGuardPolicy;

/// Log a call the guard let through. Shapes only.
pub(super) fn log_allowed(policy: &HostGuardPolicy, method: &str, namespace: &str, chars: usize) {
    log::debug!(
        "{LOG_PREFIX} allowed driver={} class={} method={method} namespace={namespace} \
         content_chars={chars}",
        policy.driver_id(),
        policy.class(),
    );
}

/// Log and publish a refusal.
///
/// Called from [`GuardPolicy::on_denied`], which the decorator's
/// `GuardPolicy::denied` runs, so every deny path audits by construction rather
/// than by each call site remembering to. `publish_global` is synchronous and a
/// no-op before the bus is initialised, so this is safe pre-boot with no
/// `#[cfg(test)]` guard — the same property `binding::build` relies on.
pub(super) fn publish_guard_denied(policy: &HostGuardPolicy, method: &str, reason: &str) {
    log::warn!(
        "{LOG_PREFIX} DENIED driver={} class={} method={method}: {reason}",
        policy.driver_id(),
        policy.class(),
    );
    BUS.publish(DomainEvent::MemoryGuardDenied {
        driver_id: policy.driver_id().to_string(),
        method: method.to_string(),
        reason: reason.to_string(),
    });
}
