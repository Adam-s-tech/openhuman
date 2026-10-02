//! [`HostGuardPolicy`] — OpenHuman's [`GuardPolicy`]: the resolved policy bundle
//! [`MemoryGuard`] and all its family decorators share.
//!
//! The decorator itself is `tinymemory_guard::GuardedProvider`; this is the
//! host half it consults — live `SecurityPolicy`, the task-local source scope,
//! the conservative sanitizer, egress disclosure and the bus audit event.
//!
//! [`MemoryGuard`]: super::MemoryGuard
//!
//! ## What is cached here, and what deliberately is not
//!
//! The *binding* facts — driver id, [`DriverClass`], the hook budgets, and the
//! configured `trust_state` — are resolved once, at bind time, and stored. They
//! cannot change without a rebind, and the binding is what constructs this
//! type.
//!
//! The [`SecurityPolicy`] is **not** cached. It is read from
//! [`live_policy::current`] on every call. That is a deliberate departure from
//! the obvious design: `MemoryBinding` is cached in a process-global,
//! workspace-keyed map for the life of the process, and `live_policy` is
//! installed *after* the first bind and hot-swapped on every autonomy change
//! (`reload_from`, `update_action_dir`). Caching an `Option<Arc<SecurityPolicy>>`
//! at construction would freeze whatever was current at first bind — in
//! practice the pre-boot `None` — and the tier check would then never fire
//! again for the rest of the process. A `RwLock` read per call is cheap and is
//! the only thing that stays correct across a hot swap.
//!
//! ## `None` policy means "no tier enforcement", never "deny"
//!
//! [`live_policy::current`] returns `None` before a session runtime has
//! installed one, which is the state roughly four thousand pre-boot unit tests
//! run in. Denying there would fail all of them at once, for the same reason
//! [`unbound_default_capabilities`] returns the full set and `core::all`'s
//! `group_allowed` returns `true` with no ambient context: denying is only ever
//! correct *after* something has actually answered.
//!
//! [`unbound_default_capabilities`]: crate::memory::binding::unbound_default_capabilities

use std::borrow::Cow;
use std::sync::Arc;

use crate::memory::api::error::MemoryError;
use crate::memory::api::provider::types::SourceScope;
use tinymemory_guard::GuardPolicy;
pub use tinymemory_guard::GUARD_DENIED_PREFIX;

use crate::config::schema::MemoryHooksConfig;
use crate::core::subsystem::DriverClass;
use crate::memory::source_scope::current_source_scope;
use crate::security::egress::emit_external_transfer;
use crate::security::egress::types::{DataKind, EgressDescriptor, EgressReason};
use crate::security::live_policy;
use crate::security::policy::ToolOperation;

/// The trust state an external driver must carry before the guard will let a
/// call reach it. Matches the value `binding::admit` requires at bind time.
pub const TRUSTED: &str = "trusted";

/// The host policy bundle every guarded call consults.
///
/// Cheap to clone-by-`Arc`: the guard holds one, and each of its family
/// decorators holds an `Arc` of the same value.
pub struct HostGuardPolicy {
    driver_id: String,
    class: DriverClass,
    hooks: MemoryHooksConfig,
    trust_state: String,
}

impl HostGuardPolicy {
    /// Build the policy for a bound driver.
    pub fn new(
        driver_id: impl Into<String>,
        class: DriverClass,
        hooks: MemoryHooksConfig,
        trust_state: impl Into<String>,
    ) -> Self {
        Self {
            driver_id: driver_id.into(),
            class,
            hooks,
            trust_state: trust_state.into(),
        }
    }

    /// How the driver was reached. A host fact, never self-reported.
    pub fn class(&self) -> DriverClass {
        self.class
    }

    /// The configured hook budgets.
    pub fn hooks(&self) -> MemoryHooksConfig {
        self.hooks
    }

    /// The configured trust state for this driver binding.
    pub fn trust_state(&self) -> &str {
        &self.trust_state
    }

    fn enforce(&self, op: ToolOperation, operation: &str) -> Result<(), MemoryError> {
        // Read live, never cached — see the module docs.
        let Some(policy) = live_policy::current() else {
            return Ok(());
        };
        policy
            .enforce_tool_operation(op, operation)
            .map_err(|reason| self.denied(operation, reason))
    }
}

impl GuardPolicy for HostGuardPolicy {
    /// The bound driver's stable id — appears in every span and audit event.
    fn driver_id(&self) -> &str {
        &self.driver_id
    }

    // ── Step 1: SecurityPolicy tier ─────────────────────────────────────────

    /// Tier check for a **read** operation.
    ///
    /// [`SecurityPolicy::enforce_tool_operation`] answers `Ok` unconditionally
    /// for [`ToolOperation::Read`] — reads are never gated by autonomy tier or
    /// by the action budget. The call is made anyway rather than skipped, so
    /// that a future tier which *does* gate reads (privacy mode, a
    /// read-quarantined tier) starts applying here without a new call site.
    ///
    /// # Errors
    ///
    /// Whatever the live policy refuses, prefixed with [`GUARD_DENIED_PREFIX`].
    ///
    /// [`SecurityPolicy::enforce_tool_operation`]: crate::security::policy::SecurityPolicy::enforce_tool_operation
    fn enforce_read(&self, operation: &str) -> Result<(), MemoryError> {
        self.enforce(ToolOperation::Read, operation)
    }

    /// Tier check for a **write** operation: the `readonly`-tier refusal, and
    /// deliberately **not** the hourly action budget.
    ///
    /// M4a routed this through [`ToolOperation::Act`], which is tier *plus*
    /// `SecurityPolicy::record_action`. That was wrong in two ways, and both
    /// bite the moment a call site migrates onto the guard (M4b):
    ///
    /// - **Double-count.** `memory/tools/store.rs` and `memory/tools/forget.rs`
    ///   already spend one budget unit each on `ToolOperation::Act`.
    ///   Re-pointing them at the guard would spend a second for the same call.
    /// - **Wrong granularity.** The budget is denominated in agent *tool
    ///   calls*. The guard sits under `MemoryCore::store`, which one bulk
    ///   ingest or one sync pass calls hundreds of times — enough to exhaust a
    ///   budget with no agent having acted at all.
    ///
    /// So the guard takes the tier half only, via
    /// `SecurityPolicy::enforce_write_tier`. That is the same shape the ~15
    /// acting tools which gate on bare `can_act()` already use
    /// (`tinytools_std::filesystem::FileWriteTool`,
    /// `tools/impl/system/python_exec.rs`, `cron/scheduler.rs`, …). Budget
    /// accounting stays where it is denominated: at the tool boundary.
    ///
    /// # Errors
    ///
    /// Whatever the live policy refuses, prefixed with [`GUARD_DENIED_PREFIX`].
    fn enforce_write(&self, operation: &str) -> Result<(), MemoryError> {
        // Read live, never cached — see the module docs.
        let Some(policy) = live_policy::current() else {
            return Ok(());
        };
        policy
            .enforce_write_tier(operation)
            .map_err(|reason| self.denied(operation, reason))
    }

    // ── Step 2: source scope as a query predicate ────────────────────────────

    /// The ambient per-turn source allowlist, in contract form.
    ///
    /// `None` is unrestricted. `Some` — including `Some` over an empty set —
    /// restricts: [`SourceScope`]'s own docs make an empty allow list deny all
    /// source-attributed content, which matches
    /// [`crate::memory::source_scope`]'s empty-allowlist semantics exactly.
    fn ambient_scope(&self) -> Option<SourceScope> {
        current_source_scope().map(SourceScope::new)
    }

    // ── Step 4: redaction ────────────────────────────────────────────────────

    /// Content on its way to the driver, redacted when the driver is external.
    ///
    /// **No-op for [`DriverClass::Embedded`], [`DriverClass::Module`] and
    /// [`DriverClass::Null`]**: nothing leaves the device, and scrubbing
    /// in-process memory writes would silently destroy the user's own data. The
    /// borrowed arm is what makes that a byte-identical pass-through rather
    /// than a re-allocation that merely happens to compare equal.
    ///
    /// For [`DriverClass::External`] the content goes through the same
    /// conservative secret/PII scrubber every other host write path uses
    /// (`memory::safety::sanitize_text`).
    ///
    /// **Do not substitute `memory::util::redact::redact` here.** That function
    /// is a *log* redactor: it returns an 8-hex-character SHA-256 prefix, so
    /// using it on egress content would not redact the write, it would delete
    /// it. `redact` belongs in log lines only.
    fn redact_outbound<'a>(&self, content: &'a str) -> Cow<'a, str> {
        match self.class {
            DriverClass::Embedded | DriverClass::Module | DriverClass::Null => {
                Cow::Borrowed(content)
            }
            DriverClass::External => {
                Cow::Owned(crate::memory::safety::sanitize_text(content).value)
            }
        }
    }

    /// [`Self::redact_outbound`] for structured payloads (KV values, document
    /// metadata), via `memory::safety::sanitize_json`. Same class rule: an
    /// unmodified pass-through for embedded and null drivers.
    fn redact_outbound_json(&self, value: serde_json::Value) -> serde_json::Value {
        match self.class {
            DriverClass::Embedded | DriverClass::Module | DriverClass::Null => value,
            DriverClass::External => crate::memory::safety::sanitize_json(&value).value,
        }
    }

    // ── Step 5: egress budget + trust state ──────────────────────────────────

    /// Per-call egress gate for an external driver.
    ///
    /// Bind-time refusal already covers the trust rule (`binding::admit` sends
    /// every `trust_state != "trusted"` external driver to the fallback), so
    /// this is the *per-call* seam: it re-checks trust so a guard constructed
    /// some other way cannot skip it, and it discloses the transfer through the
    /// existing privacy-egress machinery.
    ///
    /// `carries_content` selects the disclosed [`DataKind`]:
    /// [`DataKind::FileContent`] for calls that hand raw memory bodies across
    /// the boundary, [`DataKind::Metadata`] for the rest. Neither is a perfect
    /// fit — there is no `MemoryContent` kind today.
    ///
    /// # Errors
    ///
    /// [`MemoryError::Invalid`] when the driver is external and its
    /// `trust_state` has not been explicitly raised.
    fn check_egress(&self, method: &str, carries_content: bool) -> Result<(), MemoryError> {
        if self.class != DriverClass::External {
            return Ok(());
        }
        if self.trust_state != TRUSTED {
            return Err(self.denied(
                method,
                format!(
                    "external driver '{}' is untrusted (trust_state = \"{}\"): \
                     set trust_state = \"{TRUSTED}\" under [subsystems.memory.drivers] \
                     before memory may cross the process boundary",
                    self.driver_id, self.trust_state
                ),
            ));
        }
        emit_external_transfer(EgressDescriptor::new(
            self.driver_id.clone(),
            method.to_string(),
            true,
            EgressReason::Integration,
            if carries_content {
                vec![DataKind::FileContent]
            } else {
                vec![DataKind::Metadata]
            },
        ));
        Ok(())
    }

    // ── Step 6: char budgets ─────────────────────────────────────────────────

    /// The recall char budget, or `None` when it is disabled.
    ///
    /// `0` reads as "no budget" rather than "return nothing": a zero-length
    /// budget that silently emptied every recall would be indistinguishable
    /// from a broken driver, and the config's own default is 1000.
    fn recall_budget(&self) -> Option<usize> {
        (self.hooks.recall_max_chars > 0).then_some(self.hooks.recall_max_chars)
    }

    /// The capture char budget, or `None` when it is disabled. Same zero rule
    /// as [`Self::recall_budget`].
    fn capture_budget(&self) -> Option<usize> {
        (self.hooks.capture_max_chars > 0).then_some(self.hooks.capture_max_chars)
    }

    // `max_context_tokens` is deliberately NOT enforced here. It is a
    // context-*assembly* budget — how much recalled text an agent turn may
    // inject into a prompt — and no method on the driver contract assembles a
    // prompt. It belongs to the auto-recall hook, which is the thing that
    // actually builds the context block.

    // ── Step 7: audit ────────────────────────────────────────────────────────

    fn on_denied(&self, method: &str, reason: &str) {
        super::audit::publish_guard_denied(self, method, reason);
    }

    fn on_allowed(&self, method: &str, namespace: &str, chars: usize) {
        super::audit::log_allowed(self, method, namespace, chars);
    }
}

/// Wrap `policy` for sharing with the family decorators.
pub fn shared(policy: HostGuardPolicy) -> Arc<HostGuardPolicy> {
    Arc::new(policy)
}

#[cfg(test)]
#[path = "policy_tests.rs"]
mod tests;
