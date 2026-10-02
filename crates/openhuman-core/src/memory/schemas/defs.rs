//! [`ControllerSchema`] definitions for the `memory` namespace
//! (`openhuman.memory_*`, see `docs/specs/memory-v2.md`).

use crate::core::{ControllerSchema, FieldSchema, TypeSchema};

/// Every function of the namespace, in spec order.
pub const FUNCTIONS: [&str; 20] = [
    "engines_list",
    "engine_get",
    "engine_set",
    "recall",
    "fetch",
    "learn",
    "forget",
    "items_list",
    "conversations_get",
    "conversations_set",
    "sources_list",
    "sources_add",
    "sources_remove",
    "sources_sync",
    "context_get",
    "context_refresh",
    "context_set",
    "import_scan",
    "import_start",
    "import_status",
];

fn field(name: &'static str, ty: TypeSchema, comment: &'static str, required: bool) -> FieldSchema {
    FieldSchema {
        name,
        ty: if required {
            ty
        } else {
            TypeSchema::Option(Box::new(ty))
        },
        comment,
        required,
    }
}

fn opt(name: &'static str, ty: TypeSchema, comment: &'static str) -> FieldSchema {
    field(name, ty, comment, false)
}

fn req(name: &'static str, ty: TypeSchema, comment: &'static str) -> FieldSchema {
    field(name, ty, comment, true)
}

fn out(comment: &'static str) -> Vec<FieldSchema> {
    vec![req("result", TypeSchema::Json, comment)]
}

fn limit() -> FieldSchema {
    opt(
        "limit",
        TypeSchema::BoundedU64 { min: 1, max: 100 },
        "Most results (default 10).",
    )
}

fn filter() -> FieldSchema {
    opt("filter", TypeSchema::Json, "MetaFilter: metadata fields, kinds, sources, tags_any, observed_after/before.")
}

fn cursor() -> FieldSchema {
    opt("cursor", TypeSchema::String, "Engine cursor from a previous page.")
}

/// The schema of `function`; an unknown function gets the namespace's
/// `unknown` placeholder, as every namespace does.
pub fn schema(function: &str) -> ControllerSchema {
    let (function, description, inputs, outputs): (&'static str, &'static str, Vec<FieldSchema>, Vec<FieldSchema>) =
        match function {
            "engines_list" => ("engines_list", "List the memory engines this build offers and the active one.", vec![], out("{engines: EngineDescriptor[], active: string|null}")),
            "engine_get" => ("engine_get", "The configured memory engine, its credential and health.", vec![], out("{engine, endpoint?, has_key, status, reason?, fetch_modes}")),
            "engine_set" => (
                "engine_set",
                "Select a memory engine, optionally setting its endpoint and API key.",
                vec![
                    req("engine", TypeSchema::String, "Engine id: tinyhumans or cortexdb."),
                    opt("endpoint", TypeSchema::String, "Endpoint URL; empty clears it."),
                    opt("api_key", TypeSchema::String, "API key (cortexdb); empty removes it. Stored in the credential store, never in config."),
                ],
                out("Same as memory_engine_get."),
            ),
            "recall" => (
                "recall",
                "Ask memory a question; returns an answer with citations.",
                vec![req("question", TypeSchema::String, "The question."), filter(), limit()],
                out("{answer, citations: Citation[], model?}"),
            ),
            "fetch" => (
                "fetch",
                "Raw retrieval over stored items, filtered by metadata.",
                vec![
                    req("query", TypeSchema::String, "What to search for."),
                    opt("mode", TypeSchema::String, "keyword | vector | hybrid; limited to the engine's fetch_modes."),
                    filter(),
                    limit(),
                    cursor(),
                ],
                out("{hits: Hit[], next_cursor?}"),
            ),
            "learn" => (
                "learn",
                "Store a learning.",
                vec![
                    req("text", TypeSchema::String, "The learning."),
                    opt("kind", TypeSchema::String, "preference | fact | procedure | correction | other (default fact)."),
                    opt("confidence", TypeSchema::F64, "Confidence in 0..=1 (default 0.8)."),
                    opt("meta", TypeSchema::Json, "MemoryMeta to attach."),
                ],
                out("{id}"),
            ),
            "forget" => (
                "forget",
                "Remove items by id.",
                vec![req("ids", TypeSchema::Array(Box::new(TypeSchema::String)), "Item ids.")],
                out("{forgotten: number}"),
            ),
            "items_list" => (
                "items_list",
                "Page through stored items, newest first.",
                vec![filter(), limit(), cursor()],
                out("{items: Hit[], next_cursor?}"),
            ),
            "conversations_get" => ("conversations_get", "Conversation ingestion settings and the latest stored batches.", vec![], out("{enabled, batch_turns, idle_secs, recent}")),
            "conversations_set" => (
                "conversations_set",
                "Change conversation ingestion settings.",
                vec![
                    opt("enabled", TypeSchema::Bool, "Store conversations."),
                    opt("batch_turns", TypeSchema::BoundedU64 { min: 1, max: 100 }, "Turns per stored batch."),
                    opt("idle_secs", TypeSchema::BoundedU64 { min: 1, max: 86_400 }, "Idle seconds before a partial batch is stored."),
                ],
                out("Same as memory_conversations_get."),
            ),
            "sources_list" => ("sources_list", "List document sources with their sync state.", vec![], out("{sources: Source[]}")),
            "sources_add" => (
                "sources_add",
                "Add a document source.",
                vec![
                    req("kind", TypeSchema::String, "folder | file | link | github | rss | composio."),
                    req("target", TypeSchema::String, "Path, URL, owner/repo, feed URL or Composio toolkit."),
                    opt("label", TypeSchema::String, "Display label (default: the target)."),
                    opt("schedule_mins", TypeSchema::BoundedU64 { min: 15, max: u64::from(u32::MAX) }, "Minutes between scheduled syncs; omit for on demand only."),
                ],
                out("{source: Source}"),
            ),
            "sources_remove" => (
                "sources_remove",
                "Remove a document source.",
                vec![
                    req("id", TypeSchema::String, "Source id."),
                    opt("forget_items", TypeSchema::Bool, "Also forget everything it stored."),
                ],
                out("{removed: boolean}"),
            ),
            "sources_sync" => (
                "sources_sync",
                "Start syncing one source, or all of them.",
                vec![opt("id", TypeSchema::String, "Source id; every source when omitted.")],
                out("{started: string[]}"),
            ),
            "context_get" => ("context_get", "The compiled context.md and its settings.", vec![], out("{markdown, tokens, generated_at, interval_mins, budget_tokens, enabled}")),
            "context_refresh" => ("context_refresh", "Recompile context.md now.", vec![], out("Same as memory_context_get.")),
            "context_set" => (
                "context_set",
                "Change context.md settings.",
                vec![
                    opt("enabled", TypeSchema::Bool, "Compile and inject context.md."),
                    opt("interval_mins", TypeSchema::BoundedU64 { min: 5, max: u64::from(u32::MAX) }, "Minutes between recompiles."),
                    opt("budget_tokens", TypeSchema::BoundedU64 { min: 100, max: 32_000 }, "Token budget."),
                ],
                out("Same as memory_context_get."),
            ),
            "import_scan" => ("import_scan", "Look for a v1 memory store in this workspace.", vec![], out("{found, counts?: {documents, conversations, learnings}}")),
            "import_start" => (
                "import_start",
                "Import the v1 store into the selected engine. Uploads local data; requires consent: true.",
                vec![req("consent", TypeSchema::Bool, "Must be true.")],
                out("{state: ImportState}"),
            ),
            "import_status" => ("import_status", "Progress of the v1 import.", vec![], out("{state: ImportState}")),
            _ => (
                "unknown",
                "Unknown memory controller function.",
                vec![],
                vec![req("error", TypeSchema::String, "Lookup error details.")],
            ),
        };
    ControllerSchema {
        namespace: "memory",
        function,
        description,
        inputs,
        outputs,
    }
}
