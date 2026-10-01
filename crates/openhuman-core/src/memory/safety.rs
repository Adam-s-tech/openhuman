//! Secret and PII scrubbing for anything this host persists or hands on.
//!
//! The scrubbers live in the shared [`tinymemory_safety`] crate (credential
//! patterns, the sensitive-key classifier, the JSON depth cap, and the
//! checksum-gated multilingual national-ID PII module). This module keeps the
//! host's `memory::safety::*` import paths.
//!
//! The host uses [`host_policy`], i.e. [`tinymemory_safety::Policy::corroborated`],
//! the same policy the TinyCortex engine applies: a bare (separator-less)
//! Luhn-valid 13-19 digit run is redacted as a credit card only with a real
//! network IIN at an issued length or a card keyword nearby. Without that,
//! 13-digit epoch-millisecond timestamps and order ids in stored memory were
//! being rewritten. Separated runs (`4111 1111 1111 1111`) stay Luhn-gated.
//! The vendor default stays the strict `LuhnOnly` for other consumers, so every
//! host call goes through the wrappers below instead of the plain functions.
//!
//! Conservative by design: it prefers false positives over leaking a
//! credential into a long-lived store.

use serde_json::Value;
use tinymemory_safety::Policy;
pub use tinymemory_safety::{
    has_likely_email, has_likely_pii, has_likely_secret, SanitizationReport, Sanitized,
};

/// The single scrubbing policy for every host call site.
pub const fn host_policy() -> Policy {
    Policy::corroborated()
}

/// Scrub secrets and PII from text under [`host_policy`].
pub fn sanitize_text(value: &str) -> Sanitized<String> {
    tinymemory_safety::sanitize_text_with(value, host_policy())
}

/// Scrub secrets and PII from every string in a JSON value under [`host_policy`].
pub fn sanitize_json(value: &Value) -> Sanitized<Value> {
    tinymemory_safety::sanitize_json_with(value, host_policy())
}

/// Personal-PII detection and redaction.
pub mod pii {
    pub use tinymemory_safety::pii::{has_likely_email, has_likely_pii};

    /// Redact personal PII from text under [`super::host_policy`].
    pub fn redact_pii(text: &str) -> super::Sanitized<String> {
        tinymemory_safety::pii::redact_pii_with(text, super::host_policy())
    }
}

#[cfg(test)]
#[path = "safety_tests.rs"]
mod tests;
