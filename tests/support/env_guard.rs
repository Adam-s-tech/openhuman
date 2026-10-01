//! Process-env guard and lock shared by the integration targets.
//!
//! [`EnvVarGuard`] sets (or removes) one variable and restores the previous
//! value on drop. It does **not** serialize anything by itself: process env is
//! global, so a test that mutates it must hold [`env_lock`] (or, inside
//! `raw_coverage_all`, the aggregate's `SHARED_ENV_LOCK`) for as long as the
//! guards live, in the order lock first, guards second.
//!
//! Include with `#[path = "support/env_guard.rs"] mod env_guard;` from a
//! standalone `tests/*.rs` target, or declare it once at the root of an
//! aggregated target and `use crate::env_guard::...` from the suites.

#![allow(dead_code)]

use std::ffi::{OsStr, OsString};
use std::path::Path;
use std::sync::{Mutex, MutexGuard, OnceLock};

/// Sets or unsets one env var for the guard's lifetime and restores the value
/// that was there before when dropped.
pub struct EnvVarGuard {
    key: &'static str,
    old: Option<OsString>,
}

impl EnvVarGuard {
    /// Set `key` to `value`.
    pub fn set(key: &'static str, value: impl AsRef<OsStr>) -> Self {
        let old = std::env::var_os(key);
        std::env::set_var(key, value.as_ref());
        Self { key, old }
    }

    /// Set `key` to a filesystem path.
    pub fn set_to_path(key: &'static str, path: &Path) -> Self {
        Self::set(key, path.as_os_str())
    }

    /// Alias of [`EnvVarGuard::set_to_path`].
    pub fn set_path(key: &'static str, path: &Path) -> Self {
        Self::set_to_path(key, path)
    }

    /// Remove `key`.
    pub fn unset(key: &'static str) -> Self {
        let old = std::env::var_os(key);
        std::env::remove_var(key);
        Self { key, old }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        match &self.old {
            Some(value) => std::env::set_var(self.key, value),
            None => std::env::remove_var(self.key),
        }
    }
}

static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
static FILE_KEYRING_INIT: OnceLock<()> = OnceLock::new();

/// The process-wide env lock for this target. Poison is recovered so one
/// panicking test cannot wedge the rest of the binary.
pub fn env_lock() -> MutexGuard<'static, ()> {
    ENV_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// [`env_lock`] after pinning `OPENHUMAN_KEYRING_BACKEND=file` once per
/// process (the suites that read or write secrets need the file keyring so
/// they never touch the host's real one).
pub fn env_lock_with_file_keyring() -> MutexGuard<'static, ()> {
    FILE_KEYRING_INIT.get_or_init(|| {
        std::env::set_var("OPENHUMAN_KEYRING_BACKEND", "file");
    });
    env_lock()
}
