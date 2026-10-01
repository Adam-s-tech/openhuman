// Host-side HTTP transport for Composio's own v3 API with the user's key.
//
// What Composio's direct route sends and how it reshapes the answer is
// `tinyconnectors::client::DirectRoute` (paths, `limit=200`,
// `toolkit_versions=latest` (#3932), repeated `tags=`, blank-id filtering,
// v3-to-envelope translation). This module is only the seam the route needs
// from the host: a `Transport` that carries the key, refuses non-HTTPS bases,
// never follows redirects, and goes out through the host's proxy and TLS
// settings, plus the pre-store API-key probe's client.

#[cfg(test)]
#[path = "direct_tests.rs"]
mod tests;

mod construction;
mod http_errors;
mod transport;
mod types;

pub use types::DirectComposioClient;

// Test-only bridges: the flat `direct_tests.rs` module still expects these
// internal helpers to be reachable unqualified via `use super::*`.
#[cfg(test)]
use http_errors::{extract_api_error_message, sanitize_error_message};
#[cfg(test)]
use types::{ensure_https, is_loopback_http_url, COMPOSIO_API_BASE_V3};
