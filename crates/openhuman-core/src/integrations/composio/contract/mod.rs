//! The Composio vocabulary the host's integration code shares: toolkit
//! catalogs (curated action lists and descriptions), connected-identity
//! profiles, per-toolkit user scope preferences, provider sync bookkeeping and
//! the normalised task shape.
//!
//! These types used to be part of TinyMemory's v1 bus contract, which carried
//! them only because the memory engine ran Composio syncs. Memory v2 does not,
//! so the shapes live here, beside the integration that produces and reads
//! them. They are plain data and pure functions: no HTTP, no persistence.

pub mod catalogs;
pub mod profile;
pub mod runs;
pub mod scopes;
pub mod state;
pub mod tasks;

pub use profile::{
    canonicalize, normalize_connection_identifier, render_connected_identities_section,
    ConnectedIdentity, IdentityKind, ProviderUserProfile,
};
pub use runs::{ComposioUsage, ComposioUsageHandle, SyncOutcome, SyncReason};
pub use scopes::{
    agent_ready_toolkits, classify_unknown, find_curated, toolkit_from_slug, CuratedTool,
    ToolScope, UserScopePref,
};
pub use state::{
    extract_item_id, DailyBudget, SyncState, DEFAULT_DAILY_REQUEST_LIMIT, KV_NAMESPACE,
    STATE_NAMESPACE,
};
pub use tasks::{GithubFetchMode, NormalizedTask, TaskContainer, TaskFetchFilter, TaskKind};

pub use catalogs::{
    catalog_for_toolkit, curated_scope_for, has_native_provider, is_action_visible_with_pref,
    native_provider_sync_interval_secs, parse_sync_interval_override, sync_interval_env_var,
    toolkit_description, toolkit_has_scope, toolkit_result_notes, CAPABILITY_TOOLKITS,
    NATIVE_PROVIDERS,
};
