//! The host's half of [`ToolExposure::Deferred`](tinytools::ToolExposure):
//! which registered tools leave the wire, so the harness can advertise
//! `tool_search` in their place.
//!
//! # Why this exists
//!
//! Tool schemas are a fixed cost paid on every request. Measured on the
//! orchestrator with an empty workspace, they were 45,199 bytes against 33,808
//! bytes of system prompt; on a signed-in workspace with Composio connected
//! they reached ~112 KB. Most of that is tools the model reaches for on a
//! handful of turns a week.
//!
//! `Deferred` takes such a tool off the wire without removing the capability.
//! The tinyagents harness owns the other half of that bargain — the intrinsic
//! `tool_search` / `tool_call` bridge, the BM25 catalogue, and the pluggable
//! ranker a decision model plugs into (`tool::discover`). This host used to
//! register a `tool_search` of its own, which the harness honoured over its
//! intrinsic; that tool could *find* a deferred tool but the turn allowlist
//! never registered it, so the find was unusable. Now the host only decides
//! what is deferred and hands the harness both sets: the advertised names as
//! the wire surface and the deferred names as still-callable registrations.
//!
//! # Why not the toolpack mechanism
//!
//! Packs answer a different question. A pack is a *group* withheld by a config
//! posture, recovered through `use_skill`, and its membership is compiled in
//! precisely so config cannot move a dangerous tool out of the reviewed
//! surface. That is the right shape for compressing a belt an agent owns.
//!
//! Deferral is per-tool and is a property of the tool: `stock_quote` is rarely
//! needed whoever is running the host. The two compose — a deferred tool inside
//! a withheld pack is simply absent twice — and neither can widen the surface,
//! because both only ever subtract from a set the belt and the security policy
//! already decided.

/// The name an agent lists in `[tools] named` to opt a hand-written belt into
/// discovery. It is not a registered tool: the harness advertises its own
/// intrinsic `tool_search` whenever a run has a deferred tool, so the name in
/// a belt is a request for that, and the builder strips it from the allowlist.
pub const TOOL_SEARCH_NAME: &str = "tool_search";

pub use tinytools::{deferred_tool_names, strip_deferred_from_visible};
