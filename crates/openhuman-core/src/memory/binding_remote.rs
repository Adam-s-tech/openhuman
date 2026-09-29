//! Remote memory engines — the `External` half of [`crate::memory::binding`].
//!
//! Everything that turns `[subsystems.memory] driver = "<remote id>"` into a
//! bound [`MemoryProvider`] lives here, behind the `memory-remote` gate, so the
//! binding module itself stays a pure admission + cache seam.
//!
//! * The engines come from `tinymemory::factory` (`tinyhumans`, `supermemory`,
//!   `mem0`, `cognee`, `cortex`, `agentmemory`). The factory owns wire dialects;
//!   this module owns *where the endpoint and the credential come from*.
//! * `tinyhumans` is first-party: the endpoint is always the backend origin the
//!   installed transport reports, and the credential is a [`LiveSessionBearer`]
//!   that re-reads the host's API key / session JWT on every request, so a
//!   session refresh (or a sign-out) is picked up without rebinding.
//! * Every other engine takes its endpoint and deployment from
//!   `[subsystems.memory.drivers.<id>]` and its key from the keychain through
//!   `credential_ref = "keychain:memory-<id>"`.
//!
//! Nothing here may put a key, a `credential_ref` or an endpoint into a string
//! that reaches a log, the event bus or an RPC error: see [`scrub`].

use std::path::Path;

use crate::config::schema::MemoryDriverConfig;

/// The first-party hosted engine's id.
pub const HOSTED_ENGINE_ID: &str = tinymemory_api::drivers::TINYHUMANS_DRIVER_ID;

/// Prefix of the keychain entry a user-supplied engine key is stored under.
pub const CREDENTIAL_NAME_PREFIX: &str = "memory-";

/// The `credential_ref` value for engine `id` (`keychain:memory-<id>`).
#[must_use]
pub fn credential_ref_for(id: &str) -> String {
    format!(
        "{}:{CREDENTIAL_NAME_PREFIX}{id}",
        crate::security::credentials::credential_ref::KEYCHAIN_SCHEME
    )
}

/// Keychain partition an engine key is stored under for `workspace_dir`.
///
/// Same derivation as the wallet's: the user's openhuman dir (the workspace's
/// parent) names the partition, so two profiles never share an engine key.
#[must_use]
pub fn keyring_user_id(workspace_dir: &Path) -> String {
    workspace_dir
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .filter(|s| !s.is_empty())
        .map_or_else(
            || {
                let mut hash: u64 = 14_695_981_039_346_656_037;
                for b in workspace_dir.to_string_lossy().as_bytes() {
                    hash ^= u64::from(*b);
                    hash = hash.wrapping_mul(1_099_511_628_211);
                }
                format!("memory-path-{hash:016x}")
            },
            str::to_string,
        )
}

/// Replace every non-empty `secret` in `text` with a fixed placeholder.
///
/// Used on errors that come back from the engine layer before they reach the
/// bus or an operator-facing string: an adapter's message may name the
/// endpoint it was pointed at.
#[must_use]
pub fn scrub(text: &str, secrets: &[&str]) -> String {
    let mut out = text.to_string();
    for secret in secrets.iter().filter(|s| !s.trim().is_empty()) {
        out = out.replace(secret, "<redacted>");
    }
    out
}

/// Whether `id` is an engine this build can bind through the factory.
#[cfg(feature = "memory-remote")]
#[must_use]
pub fn is_remote_engine(id: &str) -> bool {
    tinymemory::factory::list_engines()
        .iter()
        .any(|engine| engine.id == id)
}

/// Without `memory-remote` no engine is remote.
#[cfg(not(feature = "memory-remote"))]
#[must_use]
pub fn is_remote_engine(_id: &str) -> bool {
    false
}

/// What to build: everything the factory needs except the credential's source.
#[derive(Clone)]
pub struct EngineTarget {
    /// Factory engine id.
    pub id: String,
    /// Base URL (ignored for the hosted engine).
    pub endpoint: Option<String>,
    /// `"cloud"` / `"self_hosted"`.
    pub deployment: Option<String>,
    /// A key to use as-is (an RPC's `api_key`), taking precedence over
    /// [`Self::credential_ref`]. Never logged; `Debug` is deliberately absent.
    pub api_key: Option<String>,
    /// `keychain:<name>` reference resolved when no explicit key was given.
    pub credential_ref: Option<String>,
}

impl EngineTarget {
    /// The target a persisted `[subsystems.memory.drivers.<id>]` entry names.
    #[must_use]
    pub fn from_entry(id: &str, entry: Option<&MemoryDriverConfig>) -> Self {
        Self {
            id: id.to_string(),
            endpoint: entry.and_then(|e| e.endpoint.clone()),
            deployment: entry.and_then(|e| e.deployment.clone()),
            api_key: None,
            credential_ref: entry.and_then(|e| e.credential_ref.clone()),
        }
    }
}

/// Best-effort, synchronous read of the `api_url` override for a workspace.
///
/// [`crate::memory::binding`] binds synchronously and holds no `Config`, but
/// the hosted engine's endpoint depends on the operator's `api_url` (staging,
/// self-hosted backend). The config file is looked for in the same two layouts
/// `load_config_for_workspace_with_timeout` uses; a missing or unparsable file
/// means "no override".
#[must_use]
pub fn configured_api_url(workspace_dir: &Path) -> Option<String> {
    let candidates = [
        workspace_dir.join("config.toml"),
        workspace_dir
            .parent()
            .map(|p| p.join("config.toml"))
            .unwrap_or_default(),
    ];
    let path = candidates.iter().find(|p| p.is_file())?;
    let raw = std::fs::read_to_string(path).ok()?;
    let value: toml::Value = toml::from_str(&raw).ok()?;
    value
        .get("api_url")
        .and_then(toml::Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

#[cfg(feature = "memory-remote")]
pub use imp::{build_engine, LiveSessionBearer};

#[cfg(feature = "memory-remote")]
mod imp {
    use std::path::{Path, PathBuf};
    use std::sync::Arc;

    use tinymemory::factory::{
        build_provider, BearerSource, EngineConfig, EngineCredential,
    };

    use super::{keyring_user_id, scrub, EngineTarget, HOSTED_ENGINE_ID};
    use crate::memory::api::provider::MemoryProvider;
    use crate::security::credentials::credential_ref::CredentialRef;

    /// Prefix of the error the hosted engine's bearer source reports when the
    /// host holds no credential; the RPC layer maps it to `SESSION_EXPIRED:`.
    pub const NO_SESSION_MESSAGE: &str = "SESSION_EXPIRED: no TinyHumans session";

    /// A [`BearerSource`] that reads the host's backend credential (API key,
    /// else session JWT) from live config on every request.
    pub struct LiveSessionBearer {
        workspace_dir: PathBuf,
    }

    impl LiveSessionBearer {
        /// A bearer source anchored to `workspace_dir`'s config.
        #[must_use]
        pub fn new(workspace_dir: &Path) -> Self {
            Self {
                workspace_dir: workspace_dir.to_path_buf(),
            }
        }
    }

    #[async_trait::async_trait]
    impl BearerSource for LiveSessionBearer {
        async fn bearer(&self) -> anyhow::Result<String> {
            let config =
                crate::config::ops::load_config_for_workspace_with_timeout(&self.workspace_dir)
                    .await
                    .map_err(|e| anyhow::anyhow!("{NO_SESSION_MESSAGE} (config unavailable: {e})"))?;
            match crate::security::credentials::session_support::backend_bearer_secret(&config) {
                Ok(Some(token)) if !token.trim().is_empty() => Ok(token),
                Ok(_) => Err(anyhow::anyhow!(NO_SESSION_MESSAGE)),
                Err(e) => {
                    log::debug!("[memory:remote] bearer lookup failed: {e}");
                    Err(anyhow::anyhow!("{NO_SESSION_MESSAGE} (credential lookup failed)"))
                }
            }
        }
    }

    /// Build the provider for `target`.
    ///
    /// `api_url` is the operator's backend override (`Config::api_url`), only
    /// read for the hosted engine.
    ///
    /// # Errors
    ///
    /// A scrubbed, operator-safe message: no key, `credential_ref` or endpoint.
    pub fn build_engine(
        workspace_dir: &Path,
        api_url: &Option<String>,
        target: &EngineTarget,
    ) -> Result<Arc<dyn MemoryProvider>, String> {
        let id = target.id.as_str();
        let (config, credential) = if id == HOSTED_ENGINE_ID {
            // First-party: the endpoint is the backend origin, never a config
            // value, and the credential is the live session — nothing the user
            // typed can redirect the session token elsewhere.
            let endpoint = crate::backend::require_base_url(api_url)?;
            (
                EngineConfig {
                    endpoint: Some(endpoint),
                    deployment: None,
                },
                EngineCredential::Dynamic(Arc::new(LiveSessionBearer::new(workspace_dir))),
            )
        } else {
            let credential = match (&target.api_key, &target.credential_ref) {
                (Some(key), _) if !key.trim().is_empty() => {
                    EngineCredential::Static(key.trim().to_string())
                }
                (_, Some(reference)) => {
                    let parsed = CredentialRef::parse(reference)
                        .map_err(|e| format!("engine credential reference is invalid: {e}"))?;
                    match parsed.resolve(&keyring_user_id(workspace_dir)) {
                        Ok(secret) => EngineCredential::Static(secret.expose_secret().to_string()),
                        // A key is optional for some engines; the factory
                        // refuses when this engine needs one.
                        Err(crate::security::credentials::credential_ref::CredentialRefError::NotFound) => {
                            EngineCredential::None
                        }
                        Err(e) => return Err(format!("engine credential unavailable: {e}")),
                    }
                }
                _ => EngineCredential::None,
            };
            (
                EngineConfig {
                    endpoint: target.endpoint.clone(),
                    deployment: target.deployment.clone(),
                },
                credential,
            )
        };

        let mut secrets: Vec<&str> = Vec::new();
        if let Some(endpoint) = config.endpoint.as_deref() {
            secrets.push(endpoint);
        }
        if let Some(key) = target.api_key.as_deref() {
            secrets.push(key);
        }
        build_provider(id, &config, credential)
            .map_err(|e| format!("engine '{id}' could not be built: {}", scrub(&e.to_string(), &secrets)))
    }
}
