//! Consolidated memory query tool — dispatches to the correct memory-tree
//! retrieval primitive based on the `mode` argument.
//!
//! The tools are `tinymemory-tools`' (`query`); the aliases here run them under
//! this host, so `MemoryQueryTool` keeps its name and the per-mode structs stay
//! importable. `ingest_document` is the one mode that is the host's: it writes
//! through the host's own tree-ingest path, so its tool lives here and the
//! dispatcher reaches it through [`HostMemoryTools`].

mod ingest_document;

use crate::memory::tools::host::HostMemoryTools;

pub use ingest_document::MemoryTreeIngestDocumentTool;

/// `memory_tree_cover_window` over this host.
pub type MemoryTreeCoverWindowTool =
    tinymemory_tools::query::MemoryTreeCoverWindowTool<HostMemoryTools>;
/// `memory_tree_drill_down` over this host.
pub type MemoryTreeDrillDownTool =
    tinymemory_tools::query::MemoryTreeDrillDownTool<HostMemoryTools>;
/// `memory_tree_fetch_leaves` over this host.
pub type MemoryTreeFetchLeavesTool =
    tinymemory_tools::query::MemoryTreeFetchLeavesTool<HostMemoryTools>;
/// `memory_tree_query_source` over this host.
pub type MemoryTreeQuerySourceTool =
    tinymemory_tools::query::MemoryTreeQuerySourceTool<HostMemoryTools>;
/// `memory_tree_search_entities` over this host.
pub type MemoryTreeSearchEntitiesTool =
    tinymemory_tools::query::MemoryTreeSearchEntitiesTool<HostMemoryTools>;
/// The consolidated `memory_tree` tool over this host.
pub type MemoryTreeTool = tinymemory_tools::query::MemoryTreeTool<HostMemoryTools>;
/// Alias of [`MemoryTreeTool`].
pub type MemoryQueryTool = MemoryTreeTool;
