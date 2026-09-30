//! Memory search tools — all agent-facing retrieval tools consolidated here.
//!
//! The tools themselves are `tinymemory-tools`'; these are the aliases that run
//! them under this host ([`HostMemoryTools`]), so the registration sites in
//! `tools/ops.rs` and every historical import path keep their names. Construct
//! one with `::default()`.

use crate::memory::tools::host::HostMemoryTools;

/// `memory_chunk_context` over this host.
pub type MemoryChunkContextTool = tinymemory_tools::search::MemoryChunkContextTool<HostMemoryTools>;
/// `memory_hybrid_search` over this host.
pub type MemoryHybridSearchTool = tinymemory_tools::search::MemoryHybridSearchTool<HostMemoryTools>;
/// `memory_vector_search` over this host.
pub type MemoryVectorSearchTool = tinymemory_tools::search::MemoryVectorSearchTool<HostMemoryTools>;

pub use crate::memory::tools::raw_store::{
    MemoryStoreKindsTool, MemoryStoreRawChunksTool, MemoryStoreRawSearchTool,
};

// The former agentic `walk` / `smart_walk` tools are gone — retrieval is now
// the deterministic `fast_retrieve` exposed via the `memory_tree` tool's
// `walk`/`smart_walk` modes.
pub use crate::memory::query::{
    MemoryTreeDrillDownTool, MemoryTreeFetchLeavesTool, MemoryTreeIngestDocumentTool,
    MemoryTreeQuerySourceTool, MemoryTreeSearchEntitiesTool,
};
