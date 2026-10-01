//! Secret and PII scrubbing for anything this host persists or hands on.
//!
//! The scrubbers live in the shared [`tinymemory_safety`] crate (credential
//! patterns, the sensitive-key classifier, the JSON depth cap, and the
//! checksum-gated multilingual national-ID PII module). This module keeps the
//! host's `memory::safety::*` import paths.
//!
//! The host uses the crate's default [`tinymemory_safety::Policy`], the
//! strictest one: every Luhn-valid bare 13-19 digit run is redacted as a credit
//! card. The TinyCortex engine instead demands corroboration for bare runs so
//! millisecond timestamps in stored JSON survive; that is the
//! [`tinymemory_safety::Policy::corroborated`] opt-in, not the host default.
//!
//! Conservative by design: it prefers false positives over leaking a
//! credential into a long-lived store.

pub use tinymemory_safety::{
    has_likely_email, has_likely_pii, has_likely_secret, sanitize_json, sanitize_text,
    SanitizationReport, Sanitized,
};

/// Personal-PII detection and redaction.
pub mod pii {
    pub use tinymemory_safety::pii::{has_likely_email, has_likely_pii, redact_pii};
}
