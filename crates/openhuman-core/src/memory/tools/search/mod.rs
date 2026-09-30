//! Memory search tools — all agent-facing retrieval tools consolidated here.
//!
//! New tools are defined here. Existing tools from `memory::query` and
//! `memory_store::tools` are re-exported for a unified import path.

mod chunk_context;
mod hybrid_search;
mod vector_search;

// New tools
pub use chunk_context::MemoryChunkContextTool;
pub use hybrid_search::MemoryHybridSearchTool;
pub use vector_search::MemoryVectorSearchTool;

// Re-export existing tools from memory_store::tools (previously unregistered)
pub use crate::memory::tools::raw_store::{
    MemoryStoreKindsTool, MemoryStoreRawChunksTool, MemoryStoreRawSearchTool,
};

// Re-export existing tools from memory::query. The former agentic `walk` /
// `smart_walk` tools are gone — retrieval is now the deterministic
// `fast_retrieve` exposed via the `memory_tree` tool's `walk`/`smart_walk`
// modes (see `memory_tree::retrieval::fast`).
pub use crate::memory::query::{
    MemoryTreeDrillDownTool, MemoryTreeFetchLeavesTool, MemoryTreeIngestDocumentTool,
    MemoryTreeQuerySourceTool, MemoryTreeSearchEntitiesTool,
};

#[cfg(test)]
mod zz_dump_contracts {
    use tinytools::Tool;
    #[test]
    fn dump_contracts() {
        let tools: Vec<Box<dyn Tool>> = vec![
            Box::new(super::MemoryChunkContextTool),
            Box::new(super::MemoryHybridSearchTool),
            Box::new(super::MemoryVectorSearchTool),
            Box::new(super::MemoryStoreKindsTool),
            Box::new(super::MemoryStoreRawChunksTool),
            Box::new(super::MemoryStoreRawSearchTool),
            Box::new(crate::memory::tools::tool_memory::MemoryToolsListTool),
            Box::new(crate::memory::tools::tool_memory::MemoryToolsPutTool),
            Box::new(super::MemoryTreeDrillDownTool),
            Box::new(super::MemoryTreeFetchLeavesTool),
            Box::new(super::MemoryTreeQuerySourceTool),
            Box::new(super::MemoryTreeSearchEntitiesTool),
            Box::new(crate::memory::query::MemoryTreeCoverWindowTool),
            Box::new(crate::memory::query::MemoryTreeTool),
        ];
        let mut out = serde_json::Map::new();
        for t in tools {
            out.insert(
                t.name().to_string(),
                serde_json::json!({
                    "name": t.name(),
                    "description": t.description(),
                    "parameters_schema": t.parameters_schema(),
                    "exposure": format!("{:?}", t.exposure()),
                    "permission_level": format!("{:?}", t.permission_level()),
                }),
            );
        }
        std::fs::write(
            "/tmp/claude-1000/-home-enamakel-work-openhuman/9e4bbe5d-dfd3-4283-bb6a-e34995c22b9c/scratchpad/tool_contracts.json",
            serde_json::to_string_pretty(&serde_json::Value::Object(out)).unwrap(),
        )
        .unwrap();
    }
}
