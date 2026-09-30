// Slim host-side HTTP reader for Composio's own v3 API with the user's key.
//
// Composio execution, the OAuth handoff and connection management all run in
// the `tinyconnectors` module (see `module_client`). What remains here are the
// two direct-mode reads whose request parameters the pinned module route does
// not carry: `/connected_accounts?limit=200` and `/tools` with
// `toolkit_versions=latest` (#3932).

#[cfg(test)]
#[path = "direct_tests.rs"]
mod tests;

mod connections;
mod construction;
mod discovery;
mod http_errors;
mod types;

pub use connections::ComposioConnectedAccount;
pub use types::DirectComposioClient;

// Test-only bridges: the flat `direct_tests.rs` module still expects these
// internal helpers to be reachable unqualified via `use super::*`.
#[cfg(test)]
use discovery::{ComposioToolkitRef, ComposioToolsResponse, ComposioV3Tool};
#[cfg(test)]
use http_errors::{extract_api_error_message, sanitize_error_message};
#[cfg(test)]
use types::{ensure_https, is_loopback_http_url, COMPOSIO_API_BASE_V3};
