//! OpenHuman's memory-protocol wiring: the tool vocabulary handed to the
//! upstream `MemoryProtocolMiddleware`
//! (`tinyagents_harness::middleware::MemoryProtocolMiddleware`).
//!
//! Agents are told to follow a **read-index → dedupe → write → update-index**
//! cycle around durable memory. The state machine, the corrective notes and the
//! middleware live upstream; which OpenHuman tools play which role is host
//! vocabulary and is defined here.

use std::sync::Arc;

use tinyagents_harness::middleware::{MemoryProtocolMiddleware, MemoryProtocolSpec, ModeTool};

/// Build the OpenHuman tool vocabulary for the memory protocol.
///
/// - `update_memory_md` edits either `MEMORY.md` **or** `SKILL.md`
///   (`tinytools_std::filesystem::UpdateMemoryMdTool`); only a `MEMORY.md` edit
///   reconciles the memory index, so a `SKILL.md` edit must not close the cycle.
/// - The consolidated `memory_tree` tool
///   (`crates/openhuman-core/src/memory/query/mod.rs`) is a read in every `mode`
///   except `ingest_document`, which writes a document into the tree.
/// - `remember_preference` / `save_preference` (#4458) DO persist via
///   `Memory::store`, but into dedicated preference namespaces
///   (`pinned_preferences` / `user_pref_{general,situational}`) surfaced by
///   direct system-prompt injection or per-query recall. They are NOT part of the
///   `MEMORY.md` curated wiki the archivist reconciles, so they are deliberately
///   unlisted (classified `Other`): treating them as writes would resurrect the
///   unsatisfiable "call update_memory_md" nag loop that issue removed.
pub fn memory_protocol_spec() -> Arc<MemoryProtocolSpec> {
    Arc::new(MemoryProtocolSpec {
        index_update_tool: "update_memory_md".into(),
        index_file_arg: "file".into(),
        index_file: "MEMORY.md".into(),
        // Durable mutations: create an entry, delete an entry, or ingest a
        // document into the memory tree (the split-out ingest tool).
        write_tools: [
            "memory_store",
            "memory_forget",
            "memory_tree_ingest_document",
        ]
        .map(String::from)
        .to_vec(),
        // Dedupe reads: recall/search over stored memory, or a read-only walk of
        // the memory tree.
        read_tools: [
            "memory_recall",
            "memory_vector_search",
            "memory_chunk_context",
            "memory_hybrid_search",
            "memory_tree_query_source",
            "memory_tree_search_entities",
            "memory_tree_fetch_leaves",
            "memory_tree_drill_down",
            "memory_tree_cover_window",
        ]
        .map(String::from)
        .to_vec(),
        mode_tool: Some(ModeTool {
            name: "memory_tree".into(),
            mode_arg: "mode".into(),
            write_mode: "ingest_document".into(),
        }),
        recall_tool: "memory_recall".into(),
    })
}

/// The memory-protocol middleware for an OpenHuman turn. `can_update_index`
/// states whether `update_memory_md` is actually available this turn.
pub fn memory_protocol_middleware(can_update_index: bool) -> MemoryProtocolMiddleware {
    MemoryProtocolMiddleware::with_index_update_tool(memory_protocol_spec(), can_update_index)
}
