use super::*;
use crate::security::AutonomyLevel;
use tempfile::TempDir;

// Seeding still goes through the engine handle (`UnifiedMemory` above),
// but the value types are the CONTRACT's: `tinymemory_core` re-exports
// `tinymemory_api::types::{MemoryCategory, MemoryTaint}` verbatim
// (tinymemory#18 §A1 moved them onto the contract so a second engine could
// be bound without translating). Naming them at the contract keeps the
// alias honest and is one fewer reason this file holds the crate (#5560).
use crate::memory::api::types::{MemoryCategory as EngineCategory, MemoryTaint as EngineTaint};

// ── Why `UnifiedMemory` is still here, and what replacing it costs (#5560)
//
// This whole module is `#[cfg(test)]`, so the engine is not linked into a
// production build from this file; the crate survives #5560 as a
// dev-dependency and this is the kind of fixture that keeps it. What the
// fixture cannot do is move to the bound driver in a one-line swap, and
// there are two independent reasons worth recording before someone tries.
//
// 1. **The seam is a different trait.** `test_mem` hands back
//    `Arc<dyn memory::Memory>` (= `tinymemory_api::traits::Memory`), and
//    `Arc<dyn MemoryProvider>` does not coerce to it: `MemoryProvider`'s
//    supertrait is `MemoryCore`, which is a *different* trait with taint as
//    an argument rather than a second method (see `provider/mandatory.rs`,
//    which says so at the definition). Rebinding the fixture onto
//    `memory::test_support::install_memory_driver_for_test` therefore rewrites
//    every `mem.store_with_taint(..)` / `mem.get(..)` in this module, not
//    just its two lines.
// 2. **The backend choice is load-bearing.** `FLOW_MEMORY_NAMESPACE_PREFIX`'s
//    doc above turns on `UnifiedMemory::sanitize_namespace` disagreeing with
//    `Memory::forget`'s raw `WHERE namespace = ?1`. A volatile stand-in such
//    as `tinycortex::memory::store::InMemoryMemoryStore` implements the same
//    `Memory` trait and would compile, but it does not reproduce that
//    inconsistency — so it would quietly stop testing the property the
//    prefix exists to satisfy.
//
// Every test below that touches `test_mem` is already `#[ignore]`d, and
// their ignore reason is the deeper problem: the tools resolve the *bound
// driver*, so a store seeded here is not the store they read. Making them
// coherent again is a fixture rewrite against the binding — a change worth
// making on its own, with the tests un-ignored so the result is verified,
// not folded into a dependency-shedding pass.

fn test_security() -> Arc<SecurityPolicy> {
    Arc::new(SecurityPolicy::default())
}

fn test_mem() -> (TempDir, Arc<dyn crate::memory::Memory>) {
    let tmp = TempDir::new().unwrap();
    let mem = crate::memory::tool_memory::test_helpers::MockMemory::default();
    (tmp, Arc::new(mem))
}

// ── flow_namespace / FLOW_MEMORY_NAMESPACE_PREFIX ───────────────
// (relocated from `flows::mod` — see that module's re-export comment)

#[test]
fn flow_namespace_uses_the_shared_root_prefix() {
    assert_eq!(flow_namespace("abc-123"), "flow_abc-123");
    assert!(flow_namespace("abc-123").starts_with(FLOW_MEMORY_NAMESPACE_PREFIX));
}

#[test]
fn flow_namespace_is_distinct_per_flow() {
    assert_ne!(flow_namespace("a"), flow_namespace("b"));
}

// ── FlowMemoryRecallTool ────────────────────────────────────────

#[test]
fn recall_name_and_schema() {
    let tool = FlowMemoryRecallTool::new();
    assert_eq!(tool.name(), "flow_memory_recall");
    let schema = tool.parameters_schema();
    assert!(schema["properties"]["query"].is_object());
    assert!(schema["properties"]["flow_id"].is_object());
    assert!(schema["properties"]["scope"].is_object());
}
// T-m5: a missing/invalid input param reports via `ToolResult::error`
// (an `Ok(..)` the model can read and react to in-turn), never
// `Err(anyhow!)` (a hard tool-invocation failure) — matching every other
// input-validation problem on this belt (see the scope/empty-value
// tests above, which already used this channel before the fix).
// ── FlowMemoryRememberTool ──────────────────────────────────────

#[test]
fn remember_name_and_schema() {
    let (_tmp, _mem) = test_mem();
    let tool = FlowMemoryRememberTool::new(test_security());
    assert_eq!(tool.name(), "flow_memory_remember");
    let schema = tool.parameters_schema();
    assert!(schema["properties"]["flow_id"].is_object());
    assert!(schema["properties"]["key"].is_object());
    assert!(schema["properties"]["content"].is_object());
    // No `namespace` parameter exists — the security invariant that a
    // flow can never target another namespace.
    assert!(schema["properties"]["namespace"].is_null());
    assert_eq!(tool.permission_level(), PermissionLevel::Write);
}

/// Helper: a trusted `TrustedAutomation { Workflow }` origin scoped to
/// `job_id`, the only source `flow_memory_remember` will act on since the
/// T-M2 fix.
fn trusted_workflow_origin(job_id: &str) -> AgentTurnOrigin {
    AgentTurnOrigin::TrustedAutomation {
        job_id: job_id.to_string(),
        source: TrustedAutomationSource::Workflow {
            require_approval: false,
        },
    }
}
// T-m5 (retained through the T-M2 merge): the missing-param checks run
// BEFORE the trusted-origin resolution, so they are still reachable outside
// a run and still assert the `ToolResult::error` channel rather than `Err`.