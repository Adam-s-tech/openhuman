//! Process-wide file state coordinator for cross-agent staleness detection.
//!
//! The mechanism lives in [`tinytools_std::file_state`]; this module keeps only
//! the host decision of whether the guard is on. Disable with
//! `OPENHUMAN_FILE_STATE_GUARD=0` (or `false`, `off`, `no`).

pub use tinytools_std::file_state::{
    acquire_path_lock, check_partial_read, check_stale_read, current_file_state_agent_id,
    parent_stale_files, record_read, record_write, try_global, with_file_state_agent_id,
    FileStateCoordinator, ReadStamp,
};

/// Whether `OPENHUMAN_FILE_STATE_GUARD` disables the guard.
fn guard_enabled() -> bool {
    !std::env::var("OPENHUMAN_FILE_STATE_GUARD")
        .map(|v| matches!(v.as_str(), "0" | "false" | "off" | "no"))
        .unwrap_or(false)
}

/// Initialise the process-global coordinator unless the environment disables it.
pub fn init_global() {
    let enabled = guard_enabled();
    if !enabled {
        log::debug!("[file_state] guard disabled via OPENHUMAN_FILE_STATE_GUARD");
    }
    tinytools_std::file_state::init_global(enabled);
}
