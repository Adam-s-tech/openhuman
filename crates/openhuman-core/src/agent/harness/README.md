# harness

Product shell around the tinyagents tool loop. The model/tool iteration
itself runs in `crate::agent::tinyagents` (via
`run_turn_via_tinyagents_shared`); this module owns everything OpenHuman
layers on top: sub-agent definitions, parent/child context plumbing, post-turn
memory/archival hooks, and oversized-tool-result handling.

`Agent`, its per-turn lifecycle, and transcript persistence live in
`../session_host/`, not here; this module supplies the definition/prompt data
that session host and `../subagent_host/` build turns from.

Cancellation is the tinyagents steering channel (`SteeringCommand` in
`crate::agent::tinyagents`); there is no in-house interrupt fence or
cancellation token owned here. The message queue that lets a caller steer or
follow up on an in-flight turn is `tinyagents_harness::run_queue::RunQueue`,
used here (not defined here) by `agent_graph.rs` and `fork_context.rs`.

## Responsibilities

- Define sub-agent archetypes and the definition/prompt inputs consumed by
  `../session_host/` (which owns the `Agent` struct, turn lifecycle, and
  KV-cache prefix stability) and `../subagent_host/`, which implements the
  TinyAgents sub-agent lifecycle traits directly. This module does not export
  a compatibility runner of its own.
- Define sub-agent archetypes (built-in + workspace TOML) and the task-local
  plumbing that lets a spawned tool see its parent's runtime context
  (`definition*.rs`, `builtin_definitions.rs`, `fork_context.rs`,
  `sandbox_context.rs`, `spawn_depth_context.rs`, `task_recency_context.rs`,
  `OpenHumanRunContext`).
- Run the channel/CLI turn graph (`graph.rs`) and let a built-in agent
  select a bespoke sub-agent turn graph (`agent_graph.rs`).
- Extract lessons and episodic memory after each turn as a `PostTurnHook`
  (`archivist/`).
- Offload oversized worker artifacts to the filesystem, and persist oversized
  tool results as action-workspace artifacts (`artifact_offload/`,
  `tool_result_artifacts/`).
- Run the channel/CLI turn graph (`graph.rs`) and let a built-in agent select
  a bespoke sub-agent turn graph (`agent_graph.rs`).

## Key sub-modules

| Module | Role |
| --- | --- |
| `definition*.rs`, `builtin_definitions.rs`, `definition_loader.rs` | `AgentDefinition`/`AgentDefinitionRegistry`/`SandboxMode`/`ToolScope`/`PromptSource`/`ModelSpec`; loads built-ins from `crate::agent::registry::agents` and user TOML from the workspace/home `agents/` directory. |
| `fork_context.rs`, `sandbox_context.rs`, `spawn_depth_context.rs`, `task_recency_context.rs` | Task-locals that let a spawned tool see its parent's runtime context: parent handle, sandbox mode, spawn depth, and task-recency window. `fork_context.rs` also carries the `RunQueue` handle (from `tinyagents_harness::run_queue`) down to a forked turn. |
| `graph.rs` | `run_channel_turn_via_graph` (`pub(crate)`) — the channel/CLI turn graph, thin over `run_turn_via_tinyagents_shared`; called by the `agent.run_turn` native-bus handler in `agent/bus.rs`. |
| `agent_graph.rs` | `AgentGraph` (`Default`/`Custom`), `AgentTurnRequest`, `AgentTurnResult`, `AgentTurnUsage` — per-agent sub-agent turn-graph selection consumed by `../subagent_host/`. Every built-in agent currently selects `Default`. |
| `archivist/` | `ArchivistHook` (`types.rs`, `PostTurnHook` impl in `hook_impl.rs`) — post-turn episodic insert, segment boundary detection and lifecycle, LLM recap with heuristic fallback, lesson extraction from tool failures, and raw-prose ingestion into the memory tree when `config.learning.chat_to_tree_enabled` (`boundary.rs`, `lifecycle.rs`, `recap.rs`, `resummarise.rs`, `store.rs`, `tree_ingest.rs`, `events_heuristic.rs`). |
| `artifact_offload/` | The `outputs/` / `workspace/` convention under `action_dir`: prompt half (`contract.rs`) and host policy half (`policy.rs`); mechanics (thresholds, path resolution, pointer rendering, the writer) live in `tinyagents_harness::artifacts` and are re-exported here. |
| `tool_result_artifacts/` | Persists oversized individual and aggregate tool outputs under `action_dir/artifacts/tool-results/`, replacing them with a bounded `[tool_result_preview]` envelope pointing at the full, redacted file. |
| `memory_context.rs`, `memory_context_safety.rs`, `memory_protocol.rs` | Working-memory and `[Cross-chat context]` lines surfaced into the prompt (capped by `WORKING_MEMORY_LIMIT`); trust-tier wrapping of recalled entries that came from connectors (`wrap_untrusted_for_agent`); and the read-index, dedupe, write, update-index enforcement state machine for memory-mutating tools (issue #4116). |
| `required_output.rs` | Pure validate/repair/synthesize primitives (issue #4117) that guarantee a required structured-output block (for example a `thoughts` JSON block) on every accepted turn. The orchestration that calls these lives on the session in `../session_host/turn/`. |
| `parse_wire_tests.rs` | Test-only fixtures for OpenHuman's own wire vocabulary (`inference::provider::ToolCall`, native-history JSON, OpenAI function-calling payloads). The actual `<tool_call>` parsing (tags, fenced blocks, bare JSON, `<invoke>` XML, GLM grammar, p-format) moved to the vendored `tinytools_agent` crate; nothing about recovering a tool call from model text stayed here. |
| `credentials.rs` | `scrub_credentials` — regex scrubbing of credential-shaped text (key/value secrets, AWS access-key IDs, `sk-...` keys). Applied to every tool result by `CredentialScrubMiddleware` in `agent/tinyagents/middleware/credential_scrub.rs`, installed as the innermost tool wrap so nothing downstream sees the raw secret. |

## Public surface

- `OpenHumanSessionHost`, `SessionHostBuilder`, `TurnOverrides` — re-exported from `session_host`; the
  entry point for any chat turn. External callers import these from
  `crate::agent`, which re-exports them from `session_host`.
- `run_subagent`, `SubagentRunOptions`, `SubagentRunError` — hierarchical
  sub-agent dispatch from a parent tool loop.
- `AgentDefinition`, `AgentDefinitionRegistry`, `DefinitionSource`,
  `ModelSpec`, `PromptSource`, `SandboxMode`, `ToolScope`,
  `TriggerMemoryAgent` — sub-agent archetype data model.
- `ParentExecutionContext` and its accessors (`current_parent`,
  `with_parent_context`, `current_agent_context_prepared_sources`,
  `with_agent_context_prepared_sources`) — parent runtime context for
  spawned tools.
- `current_sandbox_mode`/`with_current_sandbox_mode`,
  `current_task_recency_window`/`with_task_recency_window` — other
  task-local accessors.
- `AgentGraph`, `AgentTurnRequest`, `AgentTurnResult`, `AgentTurnUsage`.
- `LastTurnUsage`, `SubagentUsageEntry`.
- `artifact_offload::{ArtifactKind, OffloadedArtifact, new_artifact_offload, offload_oversized_result, render_artifact_offload_contract, ...}` — import via the `artifact_offload::` path (see Notes).
- `run_queue::{RunQueue, QueueMode, QueuedMessage, QueueStatus}`.

## Dependencies

- `tinyagents_harness` (vendored via `vendor/tinyagents/`) — the tool loop
  itself (`run_turn_via_tinyagents_shared`), the `run_queue` and `artifacts`
  primitives this module wraps, and the `Store` trait implemented by
  `tool_result_artifacts::ToolResultArtifactIndexStore`.
- `crate::agent::registry::agents` — built-in agent archetype TOML/prompt
  bundles loaded by `definition_loader`/`builtin_definitions`.
- `crate::security::SecurityPolicy` — workspace containment policy plumbed
  into `artifact_offload::new_artifact_offload`.
- `crate::memory` — context injection (`memory_context*`) and text
  sanitization (`tool_result_artifacts` uses `memory::safety::sanitize_text`).
- `crate::config::AgentConfig`, `crate::skills::Workflow`,
  `crate::tools::{Tool, ToolSpec}` — runtime context carried through
  `fork_context::ParentExecutionContext`.

## Used by

- `agent/mod.rs` re-exports `Agent`/`AgentBuilder` for the rest of the
  crate.
- `agent/bus.rs` serves the `agent.run_turn` native request through
  `run_channel_turn_via_graph`; channels reach the harness through that bus.
- `cron/scheduler/agent_run.rs`, `web_chat/`, `inference/local/ops/agent_chat.rs`
  (`agent_chat`) build and drive `Agent` turns directly;
  `channels/runtime/dispatch/routing.rs` consults `AgentDefinitionRegistry`/
  `ToolScope`.
- `agent/tinyagents/` middleware calls `credentials::scrub_credentials` on
  every tool result.
- `agent/orchestration/tools/*` (`spawn_subagent`, `spawn_parallel_agents`,
  `spawn_async_subagent`, `continue_subagent`, `steer_subagent`, …) call into
  `subagent_host` and the task-local context modules.

## Tests

- Unit: `harness_tests.rs`, `harness_gap_tests.rs`, plus `*_tests.rs` files
  beside each sub-module (`session/session_tests*.rs`,
  `../subagent_host/{ops_tests*,handoff_tests,extract_tool_tests,tool_prep_tests}.rs`,
  `run_queue/run_queue_tests.rs`, `tool_result_artifacts/mod_tests.rs`,
  `artifact_offload/artifact_offload_tests.rs`).
- Integration: `tests/agent_harness_public.rs`, `tests/agent_harness_e2e.rs`.

## Notes / gotchas

- `session/mod.rs` cites `docs/tinyagents-harness-migration-audit.md`, and
  `artifact_offload/mod.rs`, `agent_graph.rs`, `../subagent_host/ops/runner.rs`
  cite `plan-agents.md` (`docs/specs/plan-agents.md`) as the plan for moving
  durable state, the sub-agent graph, and offload mechanics onto TinyAgents
  primitives. Neither file is checked into this repo; the plan is not
  documented here.
- `artifact_offload` deliberately has no flat `ArtifactKind` re-export at the
  `harness` level — it would shadow `agent::artifacts::ArtifactKind` for glob
  importers. Use the `artifact_offload::` path.
- `run_queue::RunQueue::push` logs and drops `QueueMode::Interrupt` and
  `QueueMode::Parallel` messages: interrupts and forked turns are handled at
  the caller, never queued.

Related: [`gitbooks/developing/architecture/agent-harness.md`](../../../../../gitbooks/developing/architecture/agent-harness.md).
