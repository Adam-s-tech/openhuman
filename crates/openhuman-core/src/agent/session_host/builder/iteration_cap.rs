//! The single resolution point for a turn's tool-iteration cap.

use crate::agent::harness::definition::AgentDefinition;
use crate::config::AgentConfig;

/// The tool-iteration cap a turn built from `agent` runs under.
pub(crate) fn resolve_max_tool_iterations(
    agent: &AgentConfig,
    def: Option<&AgentDefinition>,
) -> usize {
    def.map_or(agent.max_tool_iterations, AgentDefinition::effective_max_iterations)
}

#[cfg(test)]
#[path = "iteration_cap_tests.rs"]
mod tests;
