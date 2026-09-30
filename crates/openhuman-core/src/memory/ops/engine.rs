//! `memory.engines_list`, `memory.engine_get`, `memory.engine_set` — choosing
//! which engine backs memory.
//!
//! The default engine is the compiled `tinymemory` module (local TinyCortex).
//! With the `memory-remote` gate the user can also bind a remote engine
//! (`tinyhumans`, `supermemory`, `mem0`, `cognee`, `cortex`, `agentmemory`)
//! through `tinymemory::factory`. This file is the RPC-facing half: validation,
//! keychain storage of the key, the `[subsystems.memory]` config write, and the
//! in-process rebind. Migration between engines is in [`super::engine_migrate`].
//!
//! Secrets: an `api_key` is written to the keychain under
//! `keychain:memory-<id>` and never appears in config, logs, events, errors or
//! any response. Responses only say whether a credential exists.

use serde::{Deserialize, Serialize};

use crate::config::schema::{Config, MemoryDriverConfig};
use crate::core::Outcome;
use crate::memory::binding::{self, MODULE_ID};
use crate::memory::binding_remote::{
    credential_ref_for, is_remote_engine, keyring_user_id, CREDENTIAL_NAME_PREFIX, HOSTED_ENGINE_ID,
};

const LOG_PREFIX: &str = "[memory:engine]";

/// Error prefix for "the hosted engine refused for lack of credits".
pub const INSUFFICIENT_CREDITS_PREFIX: &str = "INSUFFICIENT_CREDITS:";
/// Error prefix for "no valid TinyHumans session".
pub const SESSION_EXPIRED_PREFIX: &str = "SESSION_EXPIRED:";
/// Error prefix for "the engine refused this credential" (HTTP 403 — for
/// example an API key without the memory scope). Deliberately not
/// [`SESSION_EXPIRED_PREFIX`]: the app reads that one as a lapsed sign-in and
/// signs the user out, which a refused key is not.
pub const MEMORY_FORBIDDEN_PREFIX: &str = "MEMORY_FORBIDDEN:";
/// Error prefix for "the engine cannot be reached or cannot serve right now"
/// (a timeout, a refused connection, a 429 or a 5xx that outlasted retries).
pub const MEMORY_UNREACHABLE_PREFIX: &str = "MEMORY_UNREACHABLE:";

/// One selectable engine, as `memory.engines_list` reports it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EngineEntry {
    pub id: String,
    pub label: String,
    pub description: String,
    pub needs_endpoint: bool,
    pub needs_key: bool,
    pub key_optional: bool,
    pub deployments: Vec<String>,
    pub default_endpoint: Option<String>,
    pub hosted: bool,
    /// Capability families the engine is expected to advertise. Static: the
    /// bound driver's own answer is `memory.provider_status`.
    pub capabilities: Vec<String>,
}

/// Result of `memory.engines_list`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EnginesList {
    pub engines: Vec<EngineEntry>,
    pub active: String,
}

/// Result of `memory.engine_get` / `memory.engine_set`. Never carries a secret.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EngineState {
    pub driver: String,
    pub endpoint: Option<String>,
    pub deployment: Option<String>,
    pub has_credential: bool,
    pub class: String,
    pub fell_back_from: Option<String>,
    pub last_error: Option<String>,
}

/// An engine a caller asks for (`memory.engine_set`, and `to` of a migration).
///
/// No `Debug` on purpose: `api_key` is a secret.
#[derive(Clone, Deserialize)]
pub struct EngineTargetParams {
    pub driver: String,
    #[serde(default)]
    pub endpoint: Option<String>,
    #[serde(default)]
    pub deployment: Option<String>,
    #[serde(default)]
    pub api_key: Option<String>,
}

/// A validated [`EngineTargetParams`].
pub(super) struct Prepared {
    pub id: String,
    pub endpoint: Option<String>,
    pub deployment: Option<String>,
    pub api_key: Option<String>,
    /// The engine takes a user key, so its config entry carries a
    /// `credential_ref`.
    pub takes_key: bool,
}

impl std::fmt::Debug for Prepared {
    // Manual: `api_key` is a secret and must never reach `Debug` output.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Prepared")
            .field("id", &self.id)
            .field("endpoint", &self.endpoint)
            .field("deployment", &self.deployment)
            .field("api_key", &self.api_key.as_ref().map(|_| "<redacted>"))
            .field("takes_key", &self.takes_key)
            .finish()
    }
}

fn blank_to_none(value: Option<String>) -> Option<String> {
    value
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

/// The capability families each remote engine is expected to advertise, by id.
///
/// Mirrors what the adapters report at bind time (`tinymemory-remote`): the
/// three mandatory families for all, plus what each dialect adds. The module
/// engine advertises everything.
fn expected_capabilities(id: &str) -> Vec<String> {
    use tinymemory_api::capabilities::Capability as C;
    let mut caps: Vec<&'static str> = C::MANDATORY.iter().map(|c| c.as_str()).collect();
    let extra: &[C] = match id {
        MODULE_ID => {
            return crate::memory::api::capabilities::Capabilities::all()
                .iter()
                .map(|c| c.as_str().to_string())
                .collect()
        }
        "mem0" => &[C::ConversationIngest, C::Graph],
        "cognee" => &[C::Graph],
        "cortex" | "tinyhumans" => &[
            C::DocumentIngest,
            C::ConversationIngest,
            C::LearningIngest,
            C::EventIngest,
            C::Answer,
        ],
        _ => &[],
    };
    caps.extend(extra.iter().map(|c| c.as_str()));
    caps.into_iter().map(str::to_string).collect()
}

/// Every engine this build offers, module first. Never includes `null`, nor the
/// factory's in-memory `tinycortex` (not persistent).
#[must_use]
pub fn available_engines() -> Vec<EngineEntry> {
    #[cfg_attr(not(feature = "memory-remote"), allow(unused_mut))]
    let mut engines = vec![EngineEntry {
        id: MODULE_ID.to_string(),
        label: "TinyCortex (local)".to_string(),
        description: "The built-in local memory engine. Nothing leaves your device.".to_string(),
        needs_endpoint: false,
        needs_key: false,
        key_optional: false,
        deployments: vec![],
        default_endpoint: None,
        hosted: false,
        capabilities: expected_capabilities(MODULE_ID),
    }];
    #[cfg(feature = "memory-remote")]
    for d in tinymemory::factory::list_engines() {
        if d.id == tinymemory_api::drivers::TINYCORTEX_DRIVER_ID || d.id == MODULE_ID {
            continue;
        }
        engines.push(EngineEntry {
            id: d.id.to_string(),
            label: d.label.to_string(),
            description: d.description.to_string(),
            needs_endpoint: d.needs_endpoint,
            needs_key: d.needs_key,
            key_optional: d.key_optional,
            deployments: d.deployments.iter().map(|s| (*s).to_string()).collect(),
            default_endpoint: d.default_endpoint.map(str::to_string),
            hosted: d.hosted,
            capabilities: expected_capabilities(d.id),
        });
    }
    engines
}

/// The persisted driver id, with the legacy `tinycortex` alias folded onto the
/// module.
fn normalized_driver(config: &Config) -> String {
    let id = config.subsystems.memory.driver.trim();
    if id.is_empty() || id == "tinycortex" {
        MODULE_ID.to_string()
    } else {
        id.to_string()
    }
}

/// Map an engine-layer failure to the RPC error vocabulary the UI keys on:
/// `INSUFFICIENT_CREDITS:`, `SESSION_EXPIRED:`, `MEMORY_FORBIDDEN:`,
/// `MEMORY_UNREACHABLE:` or `BACKEND_UNAVAILABLE:`, else the message as is
/// (already scrubbed of keys and endpoints by the engine layer).
#[must_use]
pub fn classify_engine_error(error: &anyhow::Error) -> String {
    use crate::memory::api::error::MemoryError;
    match error.downcast_ref::<MemoryError>() {
        Some(typed) => classify_memory_error(typed),
        None => classify_engine_message(&format!("{error:#}")),
    }
}

/// [`classify_engine_error`] for a typed memory error.
#[must_use]
pub fn classify_memory_error(error: &crate::memory::api::error::MemoryError) -> String {
    use crate::memory::api::error::MemoryError;
    match error {
        // The hosted backend's 402 carries a `[USER_INSUFFICIENT_CREDITS]`
        // message prefix (`tinymemory_remote::hosted`).
        MemoryError::BudgetExceeded(message)
            if message
                .trim_start()
                .starts_with("[USER_INSUFFICIENT_CREDITS]") =>
        {
            format!("{INSUFFICIENT_CREDITS_PREFIX} the memory engine is out of credits")
        }
        MemoryError::Unauthorized(message) if is_forbidden(message) => {
            format!("{MEMORY_FORBIDDEN_PREFIX} the memory engine refused this credential")
        }
        MemoryError::Unauthorized(_) => {
            format!("{SESSION_EXPIRED_PREFIX} the memory engine rejected the session")
        }
        MemoryError::Unavailable(_) | MemoryError::Unreachable(_) | MemoryError::Timeout(_) => {
            format!("{MEMORY_UNREACHABLE_PREFIX} the memory engine is not available right now")
        }
        other => classify_engine_message(&other.to_string()),
    }
}

/// Whether an `Unauthorized` message is a 403: the credential was valid and
/// refused, as opposed to missing or lapsed. Both remote adapters render the
/// status they received ahead of any body text, as `(HTTP 403 Forbidden)`, so
/// only the message's first status counts: a 401 whose body quotes an
/// upstream 403 is still a lapsed session.
fn is_forbidden(message: &str) -> bool {
    message
        .find("(HTTP ")
        .is_some_and(|at| message[at..].starts_with("(HTTP 403 Forbidden)"))
}

/// `MemoryError`'s class tags, as its `Display` renders each variant.
const MEMORY_ERROR_CLASSES: [&str; 12] = [
    "not found: ",
    "invalid input: ",
    "budget exceeded: ",
    "path escapes workspace: ",
    "io error: ",
    "serde error: ",
    "unsupported capability: ",
    "unauthorized: ",
    "unreachable: ",
    "timed out: ",
    "unavailable: ",
    "backend failed: ",
];

/// The not-now classes: the engine could not be reached or cannot serve yet.
const NOT_NOW_CLASSES: [&str; 3] = ["unavailable: ", "unreachable: ", "timed out: "];

/// The class of the error `message` renders, with the text after its tag:
/// the outermost `MemoryError` class tag, at the start or after a caller's
/// `context: ` wrapper. A tag further in is quoted detail of that error (an
/// upstream body, say), never its class.
fn memory_error_class(message: &str) -> Option<(&'static str, &str)> {
    MEMORY_ERROR_CLASSES
        .into_iter()
        .filter_map(|class| {
            let at = if message.starts_with(class) {
                0
            } else {
                message.find(&format!(": {class}"))? + 2
            };
            Some((at, class))
        })
        .min_by_key(|&(at, _)| at)
        .map(|(at, class)| (class, &message[at + class.len()..]))
}

/// [`classify_engine_error`] for a failure that is already a string.
#[must_use]
pub fn classify_engine_message(message: &str) -> String {
    if message.contains(crate::core::observability::BACKEND_UNAVAILABLE_PREFIX)
        || message.starts_with(SESSION_EXPIRED_PREFIX)
        || message.starts_with(INSUFFICIENT_CREDITS_PREFIX)
        || message.starts_with(MEMORY_FORBIDDEN_PREFIX)
        || message.starts_with(MEMORY_UNREACHABLE_PREFIX)
    {
        return message.to_string();
    }
    if message.contains("USER_INSUFFICIENT_CREDITS") {
        return format!("{INSUFFICIENT_CREDITS_PREFIX} the memory engine is out of credits");
    }
    // Only the error's own class decides, never a class quoted in its detail.
    let class = memory_error_class(message);
    let forbidden = match class {
        Some((class, detail)) => class == "unauthorized: " && is_forbidden(detail),
        // No class tag: a bare adapter message, whose first status is its own.
        None => is_forbidden(message),
    };
    if forbidden {
        return format!("{MEMORY_FORBIDDEN_PREFIX} the memory engine refused this credential");
    }
    if message.contains(SESSION_EXPIRED_PREFIX)
        || message.contains("[UNAUTHORIZED]")
        || (message.starts_with("unauthorized:") && message.contains("re-authenticate"))
    {
        return format!("{SESSION_EXPIRED_PREFIX} no TinyHumans session");
    }
    if class.is_some_and(|(class, _)| NOT_NOW_CLASSES.contains(&class)) {
        return format!("{MEMORY_UNREACHABLE_PREFIX} the memory engine is not available right now");
    }
    message.to_string()
}

/// Reject endpoints that carry credentials or point at link-local / cloud
/// metadata addresses. Error text never echoes the endpoint (it may hold
/// userinfo).
fn validate_endpoint(endpoint: &str) -> Result<(), String> {
    use std::net::{IpAddr, Ipv6Addr};
    let Ok(parsed) = url::Url::parse(endpoint) else {
        return Err("endpoint must be an http(s) URL".to_string());
    };
    if !matches!(parsed.scheme(), "http" | "https") || parsed.host_str().is_none() {
        return Err("endpoint must be an http(s) URL".to_string());
    }
    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err(
            "endpoint must not embed credentials before the host; use the API key field"
                .to_string(),
        );
    }
    let link_local = match parsed.host() {
        Some(url::Host::Ipv4(ip)) => ip.is_link_local(),
        Some(url::Host::Ipv6(ip)) => {
            let v6: Ipv6Addr = ip;
            (v6.segments()[0] & 0xffc0) == 0xfe80
                || v6.to_ipv4_mapped().is_some_and(|v4| v4.is_link_local())
        }
        Some(url::Host::Domain(name)) => {
            let name = name.trim_end_matches('.').to_ascii_lowercase();
            name == "metadata.google.internal"
                || name.parse::<IpAddr>().is_ok_and(|ip| match ip {
                    IpAddr::V4(v4) => v4.is_link_local(),
                    IpAddr::V6(v6) => (v6.segments()[0] & 0xffc0) == 0xfe80,
                })
        }
        None => false,
    };
    if link_local {
        return Err("endpoint must not be a link-local or metadata address".to_string());
    }
    Ok(())
}

/// Validate a requested engine. No I/O.
///
/// # Errors
///
/// Unknown / non-selectable ids, a malformed endpoint or an unknown
/// deployment.
pub(super) fn prepare_target(params: EngineTargetParams) -> Result<Prepared, String> {
    let raw = params.driver.trim();
    let id = if raw == "tinycortex" { MODULE_ID } else { raw };
    if id.is_empty() {
        return Err("driver is required".to_string());
    }
    if id == MODULE_ID {
        return Ok(Prepared {
            id: MODULE_ID.to_string(),
            endpoint: None,
            deployment: None,
            api_key: None,
            takes_key: false,
        });
    }
    let entry = available_engines()
        .into_iter()
        .find(|e| e.id == id)
        .ok_or_else(|| {
            if is_remote_engine(id) {
                format!("memory engine '{id}' is not selectable")
            } else {
                format!("unknown memory engine '{id}'")
            }
        })?;

    let endpoint = blank_to_none(params.endpoint);
    if let Some(endpoint) = endpoint.as_deref() {
        validate_endpoint(endpoint)?;
    }
    let hosted = entry.hosted || id == HOSTED_ENGINE_ID;
    let deployment = blank_to_none(params.deployment);
    if let Some(deployment) = deployment.as_deref().filter(|_| !hosted) {
        if !entry.deployments.iter().any(|d| d == deployment) {
            return Err(format!(
                "deployment must be one of [{}]",
                entry.deployments.join(", ")
            ));
        }
    }
    Ok(Prepared {
        id: id.to_string(),
        // First-party engines own their endpoint; a caller-supplied one is
        // dropped so it can never redirect the session bearer.
        endpoint: if hosted { None } else { endpoint },
        deployment: if hosted { None } else { deployment },
        api_key: if entry.needs_key {
            blank_to_none(params.api_key)
        } else {
            None
        },
        takes_key: entry.needs_key && !hosted,
    })
}

/// Build the provider a prepared target names, without binding or persisting
/// anything.
#[cfg(feature = "memory-remote")]
pub(super) fn build_target_provider(
    config: &Config,
    prepared: &Prepared,
) -> Result<std::sync::Arc<dyn crate::memory::api::provider::MemoryProvider>, String> {
    use crate::memory::binding_remote::{build_engine, EngineTarget};
    let target = EngineTarget {
        id: prepared.id.clone(),
        endpoint: prepared.endpoint.clone(),
        deployment: prepared.deployment.clone(),
        api_key: prepared.api_key.clone(),
        credential_ref: prepared.takes_key.then(|| credential_ref_for(&prepared.id)),
    };
    build_engine(&config.workspace_dir, &config.api_url, &target)
        .map_err(|e| classify_engine_message(&e))
}

#[cfg(not(feature = "memory-remote"))]
pub(super) fn build_target_provider(
    _config: &Config,
    prepared: &Prepared,
) -> Result<std::sync::Arc<dyn crate::memory::api::provider::MemoryProvider>, String> {
    Err(format!(
        "memory engine '{}' is not compiled into this build",
        prepared.id
    ))
}

/// Store an engine key in the keychain (never logged).
fn store_key(config: &Config, id: &str, key: &str) -> Result<(), String> {
    use crate::security::keyring_consent::{policy, PolicyDecision};
    match policy::check_secret_access() {
        PolicyDecision::Proceed => {}
        PolicyDecision::ConsentRequired => {
            return Err("keychain access is pending user consent".to_string())
        }
        PolicyDecision::Declined => return Err("keychain access was declined".to_string()),
    }
    if !crate::security::keyring::is_available() {
        return Err("no keychain backend is available on this host".to_string());
    }
    let name = format!("{CREDENTIAL_NAME_PREFIX}{id}");
    crate::security::keyring::set(&keyring_user_id(&config.workspace_dir), &name, key)
        .map_err(|_| "could not store the engine key in the keychain".to_string())
}

fn env_pins_driver() -> bool {
    std::env::var("OPENHUMAN_MEMORY_DRIVER")
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false)
}

/// Persist `prepared` as the memory engine, then rebind in process.
///
/// Order: key → config → rebind. A failure before the config write leaves the
/// previous engine active; the key write is the only step that survives a later
/// failure, and it is harmless (an unused keychain entry).
/// One process-wide lock serialises every engine switch (`engine_set` and a
/// migration's commit), so two switches never interleave their config
/// read-modify-write or their rebind.
pub(super) static SWITCH_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// [`commit_engine_locked`] under the switch lock.
pub(super) async fn commit_engine(prepared: &Prepared) -> Result<EngineState, String> {
    let _switch = SWITCH_LOCK.lock().await;
    commit_engine_locked(prepared).await
}

/// The commit itself; the caller holds [`SWITCH_LOCK`].
///
/// Reloads the config fresh and patches only `[subsystems.memory]` (the driver
/// and that driver's entry), so a config edited while a migration ran, or a
/// stale snapshot taken when the RPC started, is never written back. `from` is
/// recomputed from that fresh config.
pub(super) async fn commit_engine_locked(prepared: &Prepared) -> Result<EngineState, String> {
    let mut config = load_config().await?;
    if env_pins_driver() {
        return Err(
            "the memory engine is pinned by OPENHUMAN_MEMORY_DRIVER; unset it to switch engines"
                .to_string(),
        );
    }
    let from = normalized_driver(&config);
    if let Some(key) = prepared.api_key.as_deref() {
        store_key(&config, &prepared.id, key)?;
    }

    let memory = &mut config.subsystems.memory;
    memory.driver = prepared.id.clone();
    if prepared.id != MODULE_ID {
        memory.drivers.insert(
            prepared.id.clone(),
            MemoryDriverConfig {
                class: Some("external".to_string()),
                transport: Some("http".to_string()),
                endpoint: prepared.endpoint.clone(),
                credential_ref: prepared.takes_key.then(|| credential_ref_for(&prepared.id)),
                // The user chose this engine in the UI: that is the trust grant.
                trust_state: "trusted".to_string(),
                deployment: prepared.deployment.clone(),
            },
        );
    }
    config
        .save()
        .await
        .map_err(|e| format!("could not save the memory engine setting: {e:#}"))?;
    log::info!(
        "{LOG_PREFIX} committed engine='{}' from='{from}' workspace={}",
        prepared.id,
        config.workspace_dir.display()
    );
    crate::memory::binding_remote::note_api_url(&config.workspace_dir, &config.api_url);
    binding::rebind(&config.workspace_dir, &from, &config.subsystems.memory).map_err(|e| {
        format!(
            "the engine switch was saved but could not be applied in this session and takes \
             effect after a restart: {e}"
        )
    })?;
    state_for(&config)
}

/// Report the engine for `config`. Resolves (and if needed binds) the driver.
pub(super) fn state_for(config: &Config) -> Result<EngineState, String> {
    let binding = binding::for_config(config)?;
    let driver = binding.driver_id().to_string();
    let entry = config.subsystems.memory.drivers.get(&driver);
    let hosted = driver == HOSTED_ENGINE_ID;
    let endpoint = if hosted {
        crate::backend::base_url(&config.api_url).ok()
    } else {
        entry.and_then(|e| e.endpoint.clone())
    };
    let has_credential = if hosted {
        crate::security::credentials::session_support::backend_bearer_secret(config)
            .ok()
            .flatten()
            .is_some_and(|t| !t.trim().is_empty())
    } else {
        entry
            .and_then(|e| e.credential_ref.as_deref())
            .and_then(|r| {
                crate::security::credentials::credential_ref::CredentialRef::parse(r).ok()
            })
            .is_some_and(|r| r.resolve(&keyring_user_id(&config.workspace_dir)).is_ok())
    };
    Ok(EngineState {
        driver,
        endpoint,
        deployment: entry.and_then(|e| e.deployment.clone()),
        has_credential,
        class: binding.class().as_str().to_string(),
        fell_back_from: binding.fallback().map(|f| f.configured_driver.clone()),
        last_error: binding.fallback().map(|f| f.reason.clone()),
    })
}

async fn load_config() -> Result<Config, String> {
    crate::config::rpc::load_config_with_timeout().await
}

/// `memory.engines_list`.
pub async fn memory_engines_list() -> Result<Outcome<EnginesList>, String> {
    let config = load_config().await?;
    let active = state_for(&config).map_or_else(|_| normalized_driver(&config), |s| s.driver);
    Ok(Outcome::new(
        EnginesList {
            engines: available_engines(),
            active,
        },
        vec![],
    ))
}

/// `memory.engine_get`.
pub async fn memory_engine_get() -> Result<Outcome<EngineState>, String> {
    let config = load_config().await?;
    Ok(Outcome::new(state_for(&config)?, vec![]))
}

/// `memory.engine_set`.
pub async fn memory_engine_set(params: EngineTargetParams) -> Result<Outcome<EngineState>, String> {
    let prepared = prepare_target(params)?;
    // One switch at a time, and never while a migration owns the switch.
    let _switch = SWITCH_LOCK.lock().await;
    if super::engine_migrate::migration_running() {
        return Err(
            "a memory migration is running; wait for it to finish or cancel it before switching"
                .to_string(),
        );
    }
    log::debug!(
        "{LOG_PREFIX} engine_set engine='{}' has_key={} has_endpoint={}",
        prepared.id,
        prepared.api_key.is_some(),
        prepared.endpoint.is_some()
    );
    if prepared.id != MODULE_ID {
        // Fail before touching the keychain or config when the engine cannot
        // even be built (no transport, missing endpoint/key, bad deployment).
        let config = load_config().await?;
        build_target_provider(&config, &prepared)?;
    }
    let state = commit_engine_locked(&prepared).await?;
    Ok(Outcome::new(state, vec![]))
}

#[cfg(test)]
#[path = "engine_tests.rs"]
mod tests;
