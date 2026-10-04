//! Whose memory a read or write belongs to: agent namespaces.
//!
//! Memory is a tree of nodes (`tinymemory::Namespace`). The root holds what
//! every agent shares — the main chat agent's learnings, synced documents,
//! imported and backfilled history. Every other agent has its own node,
//! `agent:<id>`; a sub-agent's node nests under the agent that spawned it
//! (`agent:researcher/agent:scout`), and a team member's under its team
//! (`team:<id>/agent:<id>`). Inside each node, learnings, documents and
//! conversations are kept as separate scopes by the engine.
//!
//! A [`MemoryIdentity`] is the acting agent and its node. The host scopes one
//! around every agent turn ([`within`]): the session host for top-level
//! turns, the sub-agent runner for children ([`MemoryIdentity::child`]), the
//! team runtime for members. Memory then reads it ([`current`]) to stamp what
//! an agent learns and to confine what it recalls to its [`Reach`]: its own
//! node and, unless `[memory.agents.<id>] inherit = false`, the nodes above
//! it. A sibling agent's memory is never in reach.
//!
//! The identity is never taken from model arguments.

use std::future::Future;

use tinymemory::{Namespace, Reach, Segment, SegmentKind};

use crate::config::Config;

tokio::task_local! {
    static CURRENT: MemoryIdentity;
}

/// The agent acting on memory and the node it owns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryIdentity {
    /// The agent definition id; `None` outside any agent (RPC, sync jobs).
    pub agent_id: Option<String>,
    /// The node the agent writes to.
    pub namespace: Namespace,
    /// Whether the agent also reads the nodes above its own.
    pub inherit: bool,
}

impl MemoryIdentity {
    /// The root identity: no agent, the shared node.
    #[must_use]
    pub fn root() -> Self {
        Self {
            agent_id: None,
            namespace: Namespace::ROOT,
            inherit: true,
        }
    }

    /// The identity of a top-level agent run as `agent_id`, resolved from
    /// `config` ([`namespace_for`]).
    #[must_use]
    pub fn for_agent(config: &Config, agent_id: &str) -> Self {
        let settings = config.memory.agents.get(agent_id);
        Self {
            agent_id: Some(agent_id.to_string()).filter(|id| !id.trim().is_empty()),
            namespace: namespace_for(config, agent_id, None),
            inherit: settings.is_none_or(|settings| settings.inherit),
        }
    }

    /// The identity of `agent_id` run by this agent: a sub-agent, nested
    /// under this agent's node. An agent the config pins to a node keeps it,
    /// and a root agent spawned anywhere stays at the root.
    #[must_use]
    pub fn child(&self, config: &Config, agent_id: &str) -> Self {
        let settings = config.memory.agents.get(agent_id);
        let pinned = settings.and_then(|settings| parse_namespace(settings.namespace.as_deref()));
        let namespace = match pinned {
            Some(pinned) => pinned,
            None if is_root_agent(config, agent_id) => self.namespace.clone(),
            None => nest(&self.namespace, SegmentKind::Agent, agent_id),
        };
        Self {
            agent_id: Some(agent_id.to_string()).filter(|id| !id.trim().is_empty()),
            namespace,
            inherit: settings.is_none_or(|settings| settings.inherit),
        }
    }

    /// The identity of `agent_id` as a member of `team`: its node under the
    /// team's (`team:<team>/agent:<id>`), so members share the team node.
    #[must_use]
    pub fn team_member(config: &Config, team: &str, agent_id: &str) -> Self {
        let settings = config.memory.agents.get(agent_id);
        Self {
            agent_id: Some(agent_id.to_string()).filter(|id| !id.trim().is_empty()),
            namespace: namespace_for(config, agent_id, Some(team)),
            inherit: settings.is_none_or(|settings| settings.inherit),
        }
    }

    /// What this agent reads: its node, and its ancestors when it inherits.
    #[must_use]
    pub fn reach(&self) -> Reach {
        if self.inherit {
            Reach::of(self.namespace.clone())
        } else {
            Reach::exact(self.namespace.clone())
        }
    }

    /// Where this agent's shared learnings go: the nearest node above it
    /// that is not an agent's (its team's, else the root).
    #[must_use]
    pub fn shared_namespace(&self) -> Namespace {
        self.namespace.shared_ancestor()
    }
}

/// The node `agent_id` owns, optionally as a member of `team`:
///
/// - `[memory.agents.<id>] namespace`, when set and valid;
/// - the team's node (or the root) for a root agent (`[memory]
///   root_agents`, the main chat agent by default);
/// - `team:<team>/agent:<id>`, or `agent:<id>` outside a team.
#[must_use]
pub fn namespace_for(config: &Config, agent_id: &str, team: Option<&str>) -> Namespace {
    let pinned = config
        .memory
        .agents
        .get(agent_id)
        .and_then(|settings| parse_namespace(settings.namespace.as_deref()));
    if let Some(pinned) = pinned {
        return pinned;
    }
    let base = team.map_or(Namespace::ROOT, |team| {
        nest(&Namespace::ROOT, SegmentKind::Team, team)
    });
    if agent_id.trim().is_empty() || is_root_agent(config, agent_id) {
        return base;
    }
    nest(&base, SegmentKind::Agent, agent_id)
}

/// Whether `agent_id` reads and writes the root node.
#[must_use]
pub fn is_root_agent(config: &Config, agent_id: &str) -> bool {
    config
        .memory
        .root_agents
        .iter()
        .any(|root| root == agent_id)
}

/// `namespace` with a sanitized `kind:id` segment appended; past the depth
/// limit the node itself is returned, so a runaway spawn chain shares its
/// deepest node rather than failing.
fn nest(namespace: &Namespace, kind: SegmentKind, id: &str) -> Namespace {
    namespace
        .child(Segment::sanitized(kind, id))
        .unwrap_or_else(|_| namespace.clone())
}

/// A configured namespace; an invalid one is logged and ignored.
fn parse_namespace(raw: Option<&str>) -> Option<Namespace> {
    let raw = raw?.trim();
    match raw.parse::<Namespace>() {
        Ok(namespace) => Some(namespace),
        Err(error) => {
            tracing::warn!(namespace = raw, %error, "[memory:scope] ignoring an invalid configured namespace");
            None
        }
    }
}

/// Runs `fut` as `identity`.
pub async fn within<F: Future>(identity: MemoryIdentity, fut: F) -> F::Output {
    tracing::debug!(
        agent_id = identity.agent_id.as_deref().unwrap_or("-"),
        namespace = %identity.namespace,
        "[memory:scope] acting as"
    );
    CURRENT.scope(identity, fut).await
}

/// The identity scoped around the running task, if any.
#[must_use]
pub fn current() -> Option<MemoryIdentity> {
    CURRENT.try_with(Clone::clone).ok()
}

/// Runs a top-level turn of `agent_id`: as a sub-agent of the identity
/// already in scope when another agent is running, otherwise as the agent
/// itself. A turn of the agent already in scope keeps its identity.
pub async fn within_agent<F: Future>(config: &Config, agent_id: &str, fut: F) -> F::Output {
    let identity = match current() {
        Some(outer) if outer.agent_id.as_deref() == Some(agent_id) => outer,
        Some(outer) => outer.child(config, agent_id),
        None => MemoryIdentity::for_agent(config, agent_id),
    };
    within(identity, fut).await
}

#[cfg(test)]
#[path = "scope_tests.rs"]
mod tests;
