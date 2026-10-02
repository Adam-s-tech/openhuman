//! Golden wire fixtures for the MCP server surface OpenHuman serves.
//!
//! These pin the exact bytes a client receives: initialize results, the
//! tool catalog, every JSON-RPC error shape, batching and notification
//! handling. They were captured from the implementation before the generic
//! protocol moved into `tinymcp::server`, and they must keep passing through
//! that move unchanged — a diff here is a wire change, not a refactor.
//!
//! The package version is the only substituted value: it moves on every
//! release and is not part of what these fixtures protect.

use serde_json::Value;

use super::test_support::dispatch_line;

const VERSION_PLACEHOLDER: &str = "{{CARGO_PKG_VERSION}}";

/// Tools whose presence depends on the search providers the loaded config can
/// serve. The fixture pins the config-independent base catalog.
const CONFIG_GATED_TOOLS: &[&str] = &["searxng_search", "web_search", "web_answer"];

/// `(request line, exact response line)` — `None` means no response at all.
const LINE_CASES: &[(&str, Option<&str>)] = &[
    (
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"Claude Desktop","version":"0"}}}"#,
        Some(
            r#"{"id":1,"jsonrpc":"2.0","result":{"capabilities":{"resources":{"listChanged":false,"subscribe":false},"tools":{}},"instructions":"OpenHuman MCP exposes first-level core integration: inspect the live tool catalog with core.list_tools or core.tool_instructions, inspect subagents with agent.list_subagents, run a standalone subagent with agent.run_subagent, use web_search or web_answer for live web lookups (and searxng_search when self-hosted search is enabled), and use memory.recall (answer with citations), memory.fetch or memory.list for local memory reads, and memory.learn or memory.forget to change it.","protocolVersion":"2025-06-18","serverInfo":{"name":"openhuman-core","version":"{{CARGO_PKG_VERSION}}"}}}"#,
        ),
    ),
    (
        r#"{"jsonrpc":"2.0","id":"init","method":"initialize","params":{"protocolVersion":"1999-01-01"}}"#,
        Some(
            r#"{"id":"init","jsonrpc":"2.0","result":{"capabilities":{"resources":{"listChanged":false,"subscribe":false},"tools":{}},"instructions":"OpenHuman MCP exposes first-level core integration: inspect the live tool catalog with core.list_tools or core.tool_instructions, inspect subagents with agent.list_subagents, run a standalone subagent with agent.run_subagent, use web_search or web_answer for live web lookups (and searxng_search when self-hosted search is enabled), and use memory.recall (answer with citations), memory.fetch or memory.list for local memory reads, and memory.learn or memory.forget to change it.","protocolVersion":"2025-11-25","serverInfo":{"name":"openhuman-core","version":"{{CARGO_PKG_VERSION}}"}}}"#,
        ),
    ),
    (
        r#"{"jsonrpc":"2.0","id":7,"method":"initialize"}"#,
        Some(
            r#"{"id":7,"jsonrpc":"2.0","result":{"capabilities":{"resources":{"listChanged":false,"subscribe":false},"tools":{}},"instructions":"OpenHuman MCP exposes first-level core integration: inspect the live tool catalog with core.list_tools or core.tool_instructions, inspect subagents with agent.list_subagents, run a standalone subagent with agent.run_subagent, use web_search or web_answer for live web lookups (and searxng_search when self-hosted search is enabled), and use memory.recall (answer with citations), memory.fetch or memory.list for local memory reads, and memory.learn or memory.forget to change it.","protocolVersion":"2025-11-25","serverInfo":{"name":"openhuman-core","version":"{{CARGO_PKG_VERSION}}"}}}"#,
        ),
    ),
    (
        r#"{"jsonrpc":"2.0","id":"abc","method":"ping"}"#,
        Some(r#"{"id":"abc","jsonrpc":"2.0","result":{}}"#),
    ),
    (
        r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
        None,
    ),
    (
        r#"{"jsonrpc":"2.0","method":"notifications/cancelled","params":{"requestId":1}}"#,
        None,
    ),
    (r#"{"jsonrpc":"2.0","method":"notifications/other"}"#, None),
    (
        r#"[]"#,
        Some(
            r#"{"error":{"code":-32600,"data":"batch must not be empty","message":"Invalid Request"},"id":null,"jsonrpc":"2.0"}"#,
        ),
    ),
    (
        r#"[{"jsonrpc":"2.0","id":1,"method":"ping"},{"jsonrpc":"2.0","method":"notifications/initialized"},42,{"jsonrpc":"2.0","id":3,"method":"nope"}]"#,
        Some(
            r#"[{"id":1,"jsonrpc":"2.0","result":{}},{"error":{"code":-32600,"data":"message must be a JSON object","message":"Invalid Request"},"id":null,"jsonrpc":"2.0"},{"error":{"code":-32601,"data":"unsupported MCP method `nope`","message":"Method not found"},"id":3,"jsonrpc":"2.0"}]"#,
        ),
    ),
    (
        r#"[{"jsonrpc":"2.0","method":"notifications/initialized"}]"#,
        None,
    ),
    (
        r#"42"#,
        Some(
            r#"{"error":{"code":-32600,"data":"message must be a JSON object","message":"Invalid Request"},"id":null,"jsonrpc":"2.0"}"#,
        ),
    ),
    (
        r#"{"jsonrpc":"2.0","id":1.5,"method":"ping"}"#,
        Some(
            r#"{"error":{"code":-32600,"data":"id must be a string or integer","message":"Invalid Request"},"id":null,"jsonrpc":"2.0"}"#,
        ),
    ),
    (
        r#"{"jsonrpc":"2.0","id":true,"method":"ping"}"#,
        Some(
            r#"{"error":{"code":-32600,"data":"id must be a string or integer","message":"Invalid Request"},"id":null,"jsonrpc":"2.0"}"#,
        ),
    ),
    (
        r#"{"jsonrpc":"2.0","id":null,"method":"ping"}"#,
        Some(
            r#"{"error":{"code":-32600,"data":"id must be a string or integer","message":"Invalid Request"},"id":null,"jsonrpc":"2.0"}"#,
        ),
    ),
    (
        r#"{"id":1,"method":"ping"}"#,
        Some(
            r#"{"error":{"code":-32600,"data":"jsonrpc must be \"2.0\"","message":"Invalid Request"},"id":1,"jsonrpc":"2.0"}"#,
        ),
    ),
    (
        r#"{"jsonrpc":"1.0","id":1,"method":"ping"}"#,
        Some(
            r#"{"error":{"code":-32600,"data":"jsonrpc must be \"2.0\"","message":"Invalid Request"},"id":1,"jsonrpc":"2.0"}"#,
        ),
    ),
    (
        r#"{"jsonrpc":"2.0","id":1}"#,
        Some(
            r#"{"error":{"code":-32600,"data":"method must be a string","message":"Invalid Request"},"id":1,"jsonrpc":"2.0"}"#,
        ),
    ),
    (
        r#"{"jsonrpc":"2.0","id":1,"method":5}"#,
        Some(
            r#"{"error":{"code":-32600,"data":"method must be a string","message":"Invalid Request"},"id":1,"jsonrpc":"2.0"}"#,
        ),
    ),
    (
        r#"{"jsonrpc":"2.0","id":1,"method":"prompts/list"}"#,
        Some(
            r#"{"error":{"code":-32601,"data":"unsupported MCP method `prompts/list`","message":"Method not found"},"id":1,"jsonrpc":"2.0"}"#,
        ),
    ),
    (
        r#"{not-json"#,
        Some(
            r#"{"error":{"code":-32700,"data":"key must be a string at line 1 column 2","message":"Parse error"},"id":null,"jsonrpc":"2.0"}"#,
        ),
    ),
    (
        r#"{"jsonrpc":"2.0","id":1,"method":"tools/call"}"#,
        Some(
            r#"{"error":{"code":-32602,"data":"tools/call params must be an object","message":"Invalid params"},"id":1,"jsonrpc":"2.0"}"#,
        ),
    ),
    (
        r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":[]}"#,
        Some(
            r#"{"error":{"code":-32602,"data":"tools/call params must be an object","message":"Invalid params"},"id":1,"jsonrpc":"2.0"}"#,
        ),
    ),
    (
        r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{}}"#,
        Some(
            r#"{"error":{"code":-32602,"data":"tools/call params.name must be a non-empty string","message":"Invalid params"},"id":1,"jsonrpc":"2.0"}"#,
        ),
    ),
    (
        r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"   "}}"#,
        Some(
            r#"{"error":{"code":-32602,"data":"tools/call params.name must be a non-empty string","message":"Invalid params"},"id":1,"jsonrpc":"2.0"}"#,
        ),
    ),
    (
        r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"memory.recall","arguments":[1,2]}}"#,
        Some(
            r#"{"error":{"code":-32602,"data":"tools/call params.arguments: tool arguments must be a JSON object, not an array","message":"Invalid params"},"id":1,"jsonrpc":"2.0"}"#,
        ),
    ),
    (
        r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"memory.recall","arguments":"not json"}}"#,
        Some(
            r#"{"error":{"code":-32602,"data":"tools/call params.arguments: tool arguments must be a JSON object, not a string that is not JSON","message":"Invalid params"},"id":1,"jsonrpc":"2.0"}"#,
        ),
    ),
    (
        r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"memory.recall","arguments":7}}"#,
        Some(
            r#"{"error":{"code":-32602,"data":"tools/call params.arguments: tool arguments must be a JSON object, not a number","message":"Invalid params"},"id":1,"jsonrpc":"2.0"}"#,
        ),
    ),
    (
        r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"memory.recall","arguments":{}}}"#,
        Some(
            r#"{"error":{"code":-32602,"data":"missing required argument `question`","message":"Invalid params"},"id":1,"jsonrpc":"2.0"}"#,
        ),
    ),
    (
        r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"memory.recall","arguments":"{}"}}"#,
        Some(
            r#"{"error":{"code":-32602,"data":"missing required argument `question`","message":"Invalid params"},"id":1,"jsonrpc":"2.0"}"#,
        ),
    ),
    (
        r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"memory.recall"}}"#,
        Some(
            r#"{"error":{"code":-32602,"data":"missing required argument `question`","message":"Invalid params"},"id":1,"jsonrpc":"2.0"}"#,
        ),
    ),
    (
        r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"memory.recall","arguments":{"question":"x","bogus":1}}}"#,
        Some(
            r#"{"error":{"code":-32602,"data":"unexpected argument `bogus`","message":"Invalid params"},"id":1,"jsonrpc":"2.0"}"#,
        ),
    ),
    (
        r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"no.such_tool","arguments":{}}}"#,
        Some(
            r#"{"error":{"code":-32602,"data":"unknown MCP tool `no.such_tool`","message":"Invalid params"},"id":1,"jsonrpc":"2.0"}"#,
        ),
    ),
    (
        r#"{"jsonrpc":"2.0","id":12,"method":"resources/read","params":{"uri":"openhuman://prompts/agents/does_not_exist"}}"#,
        Some(
            r#"{"error":{"code":-32002,"data":"no resource with uri `openhuman://prompts/agents/does_not_exist`","message":"Resource not found"},"id":12,"jsonrpc":"2.0"}"#,
        ),
    ),
    (
        r#"{"jsonrpc":"2.0","id":13,"method":"resources/read","params":{}}"#,
        Some(
            r#"{"error":{"code":-32602,"data":"resources/read params.uri must be a non-empty string","message":"Invalid params"},"id":13,"jsonrpc":"2.0"}"#,
        ),
    ),
    (
        r#"{"jsonrpc":"2.0","id":13,"method":"resources/read","params":{"uri":"  "}}"#,
        Some(
            r#"{"error":{"code":-32602,"data":"resources/read params.uri must be a non-empty string","message":"Invalid params"},"id":13,"jsonrpc":"2.0"}"#,
        ),
    ),
    (
        r#"{"jsonrpc":"2.0","id":13,"method":"resources/read"}"#,
        Some(
            r#"{"error":{"code":-32602,"data":"resources/read params.uri must be a non-empty string","message":"Invalid params"},"id":13,"jsonrpc":"2.0"}"#,
        ),
    ),
    (
        r#"{"jsonrpc":"2.0","id":14,"method":"resources/templates/list"}"#,
        Some(r#"{"id":14,"jsonrpc":"2.0","result":{"resourceTemplates":[]}}"#),
    ),
    (
        r#"{"jsonrpc":"2.0","id":15,"method":"resources/templates/list","params":{"cursor":"x"}}"#,
        Some(r#"{"id":15,"jsonrpc":"2.0","result":{"resourceTemplates":[]}}"#),
    ),
];

const TOOLS_LIST_BASE: &str = r#"{"id":2,"jsonrpc":"2.0","result":{"tools":[{"annotations":{"openWorldHint":false,"readOnlyHint":true},"description":"List the live core agent tool catalog that OpenHuman exposes to its orchestrator session.","inputSchema":{"additionalProperties":false,"properties":{},"type":"object"},"name":"core.list_tools","title":"List Core Tools"},{"annotations":{"openWorldHint":false,"readOnlyHint":true},"description":"Emit the markdown tool-use instructions block that OpenHuman injects into prompt-guided agents.","inputSchema":{"additionalProperties":false,"properties":{},"type":"object"},"name":"core.tool_instructions","title":"Get Tool Instructions"},{"annotations":{"openWorldHint":false,"readOnlyHint":true},"description":"List registered sub-agent definitions that the core can dispatch for specialized work.","inputSchema":{"additionalProperties":false,"properties":{},"type":"object"},"name":"agent.list_subagents","title":"List Subagents"},{"annotations":{"destructiveHint":true,"idempotentHint":false,"openWorldHint":true,"readOnlyHint":false},"description":"Run a registered OpenHuman sub-agent directly from the core and return its final response.","inputSchema":{"additionalProperties":false,"properties":{"agent_id":{"description":"Registered sub-agent id (for example `planner`, `code_executor`, `critic`).","type":"string"},"prompt":{"description":"Task prompt for the sub-agent. Include the context it needs because this is a fresh session.","type":"string"}},"required":["agent_id","prompt"],"type":"object"},"name":"agent.run_subagent","title":"Run Subagent"},{"annotations":{"openWorldHint":false,"readOnlyHint":true},"description":"Keyword-search OpenHuman's local memory tree and return matching chunks ordered by recency. Every token in the query must appear in the stored chunk preview (ASCII case-insensitive, any order); punctuation between tokens does not matter. Results are preview-based, so zero hits do not prove that content is absent; try fewer/shorter tokens or `memory.recall` (semantic).","inputSchema":{"additionalProperties":false,"properties":{"k":{"description":"Maximum chunks to return. Defaults to 10; capped at 50.","maximum":50,"minimum":1,"type":"integer"},"query":{"description":"Keywords; each must appear in the stored chunk preview (any order). Prefer a few short, distinctive tokens over long exact phrases; zero hits do not prove absence.","minLength":1,"type":"string"}},"required":["query"],"type":"object"},"name":"memory.recall","title":"Search Memory"},{"annotations":{"openWorldHint":false,"readOnlyHint":true},"description":"Semantically recall local memory-tree chunks relevant to a natural-language query.","inputSchema":{"additionalProperties":false,"properties":{"k":{"description":"Maximum chunks to return. Defaults to 10; capped at 50.","maximum":50,"minimum":1,"type":"integer"},"query":{"description":"Natural-language query to embed and rerank against memory summaries.","minLength":1,"type":"string"}},"required":["query"],"type":"object"},"name":"memory.recall","title":"Recall Memory"},{"annotations":{"openWorldHint":false,"readOnlyHint":true},"description":"Read one memory-tree chunk by id. Use this to inspect the source text behind search or recall results.","inputSchema":{"additionalProperties":false,"properties":{"chunk_id":{"description":"Chunk id returned by memory.search or memory.recall.","type":"string"}},"required":["chunk_id"],"type":"object"},"name":"tree.read_chunk","title":"Read Memory Chunk"},{"annotations":{"openWorldHint":false,"readOnlyHint":true},"description":"Paginated listing of memory-tree chunks in reverse-chronological order, with optional filters by source kind, source id, entity id, time window, and token-AND keyword. Every token must appear in the stored chunk preview (ASCII case-insensitive, any order; punctuation does not matter). Use this when the user wants to enumerate (\"what's recent in my Gmail\", \"show me everything from last week about Alice\") rather than search by query. Returns chunks plus a total match count for pagination.","inputSchema":{"additionalProperties":false,"properties":{"entity_ids":{"description":"Restrict to chunks referencing any of these canonical entity ids (e.g. `person:Alice`, `email:alice@example.com`). Use `tree.top_entities` to discover these.","items":{"type":"string"},"type":"array"},"k":{"description":"Maximum chunks per page. Defaults to 10; capped at 50.","maximum":50,"minimum":1,"type":"integer"},"offset":{"description":"Pagination offset (number of rows to skip). Defaults to 0.","minimum":0,"type":"integer"},"query":{"description":"Keywords matched against the stored chunk preview; every token must appear (ASCII case-insensitive, any order; punctuation does not matter). Zero hits do not prove absence.","minLength":1,"type":"string"},"since_ms":{"description":"Inclusive lower bound on chunk timestamp, in milliseconds since Unix epoch.","minimum":0,"type":"integer"},"source_ids":{"description":"Restrict to specific logical source ids (e.g. a Slack channel id). Use `tree.list_sources` to discover these.","items":{"type":"string"},"type":"array"},"source_kinds":{"description":"Restrict to one or more source kinds (e.g. `email`, `chat`, `document`). Omit to include all kinds.","items":{"type":"string"},"type":"array"},"until_ms":{"description":"Inclusive upper bound on chunk timestamp, in milliseconds since Unix epoch.","minimum":0,"type":"integer"}},"required":[],"type":"object"},"name":"tree.browse","title":"Browse Memory"},{"annotations":{"openWorldHint":false,"readOnlyHint":true},"description":"List the most-referenced canonical entities (people, organizations, topics, emails) across the local memory tree. Call this for entity discovery before drilling in with `tree.browse` (passing `entity_ids`) or `memory.search`. Returns entities ordered by reference count.","inputSchema":{"additionalProperties":false,"properties":{"k":{"description":"Maximum entities to return. Defaults to 10; capped at 50.","maximum":50,"minimum":1,"type":"integer"},"kind":{"description":"Restrict to a single entity kind (`person`, `email`, `topic`, `org`, …). Omit to span all kinds.","minLength":1,"type":"string"}},"required":[],"type":"object"},"name":"tree.top_entities","title":"Top Memory Entities"},{"annotations":{"openWorldHint":false,"readOnlyHint":true},"description":"List every distinct ingest source (Gmail account, Slack channel, Notion workspace, email thread, …) that has data in the memory tree, with chunk counts and last-activity timestamps. Use this when the user asks \"what data sources do I have\" or to discover source ids to pass into `tree.browse`.","inputSchema":{"additionalProperties":false,"properties":{"user_email_hint":{"description":"When provided, the user's own email is stripped from email-thread display names so the other party shows up instead. Optional.","minLength":1,"type":"string"}},"required":[],"type":"object"},"name":"tree.list_sources","title":"List Memory Sources"},{"annotations":{"destructiveHint":true,"idempotentHint":true,"openWorldHint":false,"readOnlyHint":false},"description":"Create a new memory document from content. The document is stored in the specified namespace (default `mcp`) and can be retrieved via `memory.search` or `memory.recall`.","inputSchema":{"additionalProperties":false,"properties":{"content":{"description":"The text content to store as a memory document.","minLength":1,"type":"string"},"namespace":{"description":"Namespace to store the document in. Defaults to `mcp` when omitted.","minLength":1,"type":"string"},"tags":{"description":"Optional tags for categorisation and filtering.","items":{"type":"string"},"type":"array"},"title":{"description":"Human-readable title for the memory document.","minLength":1,"type":"string"}},"required":["title","content"],"type":"object"},"name":"memory.store","title":"Store Memory"},{"annotations":{"destructiveHint":true,"idempotentHint":true,"openWorldHint":false,"readOnlyHint":false},"description":"Append a note to an existing memory chunk by storing a linked annotation document. The note references the original chunk_id for provenance and can be retrieved alongside it.","inputSchema":{"additionalProperties":false,"properties":{"chunk_id":{"description":"ID of the memory chunk to annotate. Use an ID from memory.search or memory.recall results.","minLength":1,"type":"string"},"note_text":{"description":"The note text to attach to the chunk.","minLength":1,"type":"string"}},"required":["chunk_id","note_text"],"type":"object"},"name":"memory.note","title":"Annotate Memory Chunk"},{"annotations":{"destructiveHint":true,"idempotentHint":true,"openWorldHint":false,"readOnlyHint":false},"description":"Apply one or more category tags to an existing memory chunk. Stored as an upsertable tag-record document linked to the target chunk_id, so re-tagging the same chunk replaces the prior tag set rather than accumulating duplicate annotations. Differs from `memory.note` in that the payload is a categorical label list — queryable via the document `tags` field — rather than free-form text.","inputSchema":{"additionalProperties":false,"properties":{"chunk_id":{"description":"ID of the memory chunk to tag. Use an ID from `memory.search`, `memory.recall`, or `tree.browse` results.","minLength":1,"type":"string"},"tags":{"description":"One or more category labels to attach (e.g. `[\"todo\", \"q3-planning\"]`). Re-tagging the same chunk replaces the prior tag set; supply the complete desired set on each call.","items":{"minLength":1,"type":"string"},"minItems":1,"type":"array"}},"required":["chunk_id","tags"],"type":"object"},"name":"tree.tag","title":"Tag Memory Chunk"}]}}"#;

/// The resource catalog's ungated head. The tail depends on the `flows` and
/// `skills` features, so it is checked by shape rather than by bytes.
const RESOURCES_LIST_PREFIX: &str = r#"{"id":10,"jsonrpc":"2.0","result":{"resources":[{"description":"Core agent identity definition (IDENTITY.md).","mimeType":"text/markdown","name":"Agent Identity","uri":"openhuman://prompts/identity"},{"description":"Core agent personality and values (SOUL.md).","mimeType":"text/markdown","name":"Agent Soul","uri":"openhuman://prompts/soul"},{"description":"Core user-profile context injected into every session (USER.md).","mimeType":"text/markdown","name":"User Context","uri":"openhuman://prompts/user"},"#;

fn golden(expected: &str) -> String {
    expected.replace(VERSION_PLACEHOLDER, env!("CARGO_PKG_VERSION"))
}

#[tokio::test]
async fn every_line_case_answers_with_its_golden_bytes() {
    for (request, expected) in LINE_CASES {
        let actual = dispatch_line(request).await;
        assert_eq!(
            actual,
            expected.map(golden),
            "wire drift for request {request}"
        );
    }
}

#[tokio::test]
async fn tools_list_answers_with_the_golden_base_catalog() {
    let line = dispatch_line(r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#)
        .await
        .expect("tools/list answers");
    let mut response: Value = serde_json::from_str(&line).expect("json response");
    response["result"]["tools"]
        .as_array_mut()
        .expect("tools array")
        .retain(|tool| !CONFIG_GATED_TOOLS.contains(&tool["name"].as_str().expect("tool name")));
    assert_eq!(response.to_string(), TOOLS_LIST_BASE);
}

#[tokio::test]
async fn resources_list_answers_with_the_golden_catalog_shape() {
    let line = dispatch_line(r#"{"jsonrpc":"2.0","id":10,"method":"resources/list"}"#)
        .await
        .expect("resources/list answers");
    assert!(
        line.starts_with(RESOURCES_LIST_PREFIX),
        "resources/list head drifted: {line}"
    );
    let response: Value = serde_json::from_str(&line).expect("json response");
    for resource in response["result"]["resources"].as_array().expect("array") {
        let mut keys = resource
            .as_object()
            .expect("resource object")
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        keys.sort();
        assert_eq!(keys, ["description", "mimeType", "name", "uri"]);
        assert_eq!(resource["mimeType"], "text/markdown");
    }
}

#[tokio::test]
async fn resources_read_answers_with_the_embedded_prompt() {
    let line = dispatch_line(
        r#"{"jsonrpc":"2.0","id":11,"method":"resources/read","params":{"uri":"openhuman://prompts/identity"}}"#,
    )
    .await
    .expect("resources/read answers");
    let expected = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 11,
        "result": {
            "contents": [{
                "uri": "openhuman://prompts/identity",
                "mimeType": "text/markdown",
                "text": include_str!("../../agent/prompts/IDENTITY.md"),
            }]
        }
    });
    assert_eq!(line, expected.to_string());
}
