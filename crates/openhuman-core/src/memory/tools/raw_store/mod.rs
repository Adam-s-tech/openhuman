//! Raw search/retrieve tools surfaced to the agent harness.
//!
//! These tools expose the storage layer directly — no policy, no scoring
//! beyond what the underlying backend already applies. They are
//! `tinymemory-tools`' (`raw_store`); these aliases run them under this host.

use crate::memory::tools::host::HostMemoryTools;

/// `memory_store_kinds` over this host.
pub type MemoryStoreKindsTool = tinymemory_tools::raw_store::MemoryStoreKindsTool<HostMemoryTools>;
/// `memory_store_raw_chunks` over this host.
pub type MemoryStoreRawChunksTool =
    tinymemory_tools::raw_store::MemoryStoreRawChunksTool<HostMemoryTools>;
/// `memory_store_raw_search` over this host.
pub type MemoryStoreRawSearchTool =
    tinymemory_tools::raw_store::MemoryStoreRawSearchTool<HostMemoryTools>;
