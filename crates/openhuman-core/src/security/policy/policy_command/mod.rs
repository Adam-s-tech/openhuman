//! Shell-command classification internals used by [`super::SecurityPolicy`]'s
//! command-risk and allowlist gates.
//!
//! Split by responsibility:
//! - [`env_guard`]: the dangerous inline env-assignment names.
//! - [`command_name`]: the "executes arbitrary code" bases.
//!
//! The lexical scanning (quote-aware segment splitting, unquoted-character
//! detection, quoted-heredoc blanking, basename and env-assignment parsing) is
//! `tinybox_core::shell::scan`.
//! - [`classification`]: read/write/network/install/destructive bucket lists
//!   and the hidden-execution structural guard.

mod classification;
mod command_name;
mod env_guard;

pub(super) use classification::{classify_segment, has_hidden_execution};
pub(super) use command_name::is_command_executor;
pub(super) use env_guard::has_dangerous_env_prefix;
