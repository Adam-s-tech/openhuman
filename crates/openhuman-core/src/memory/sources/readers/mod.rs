//! Source readers: the [`SourceReader`] trait plus one implementation per
//! [`SourceKind`], all of which live in [`tinymemory_sources::readers`].
//!
//! A reader knows how to *list* the items available in a source and *read* the
//! content of one item, so ingestion can be driven uniformly across kinds. The
//! trait takes the workspace directory as a `&Path` and reports
//! `tinymemory_api::error::MemoryError`; the RPC handlers that call it pass
//! `config.workspace_dir` and stringify the error.
//!
//! # `reader_for_request` hands out network readers, and that is a real decision
//!
//! [`tinymemory_sources::readers::reader_for`] returns `None` for the network
//! kinds **on purpose**: a network reader is meant to be "constructed
//! explicitly by a caller that has already decided the fetch is allowed; it is
//! never handed out by the kind-dispatch that the workspace sync loop drives on
//! a timer", which is what keeps the host in charge of egress, OAuth and cost
//! budgeting.
//!
//! [`reader_for_request`] hands out all seven, because the callers here are RPC
//! handlers acting on an explicit user request — a `memory_sources_*` call
//! naming one source id — and not a timer. **Do not reuse it from a polling
//! loop.** If one ever needs a reader, it should reach for the local kinds
//! through the crate's dispatch and construct a network reader deliberately, so
//! the decision stays visible.

pub use tinymemory_sources::readers::{
    composio, conversation, folder, github, reader_for_request, rss, twitter, web_page,
    SourceReader,
};
