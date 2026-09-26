//! JSON-RPC 2.0 for OpenHuman: in-process dispatch plus the HTTP transport.
//!
//! | Module | Owns |
//! |---|---|
//! | `invoke` | [`invoke_method`] — the dispatch every transport and the CLI call |
//! | `classify` | session-expiry and Sentry-routing classification of failures |
//! | `server` | the `run_server*` shims over `CoreBuilder` |
//! | `http` | the axum router and handlers (`http-server` feature only) |
//!
//! The wire contract — envelopes, params shape, the browser-origin
//! allowlist — lives in [`crate::rpc`] (`crates/openhuman-rpc`), shared with
//! the app and TUI clients. Runtime boot (event-bus subscribers, ledgers, the
//! approval gate) lives in [`crate::core::runtime`].

mod classify;
// The axum server surface — router, handlers, middleware, extractors, SSE — is
// exclusive to the `http-server` feature (#5048). The dispatch surface and the
// `run_server*` CoreBuilder shims stay compiled; a slim build drives dispatch
// over the CLI / native path without binding a listener.
#[cfg(feature = "http-server")]
pub(crate) mod http;
mod invoke;
mod server;

pub use crate::rpc::parse_json_params;
#[cfg(feature = "http-server")]
pub use http::{build_core_http_router, rpc_handler};
pub use invoke::{default_state, invoke_method};
#[cfg(feature = "http-server")]
pub(crate) use server::{core_host, core_port};
pub use server::{
    run_server, run_server_embedded, run_server_embedded_with_ready, run_server_headless,
    EmbeddedReadySignal,
};
