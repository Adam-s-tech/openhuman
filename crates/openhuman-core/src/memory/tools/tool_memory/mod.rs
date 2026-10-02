//! Agent tools for reading and writing tool-scoped memory.
//!
//! The agent uses these to introspect what rules / learnings exist for a
//! specific tool and to record new ones discovered mid-session. They are
//! `tinymemory-tools`' (`tool_memory`); these aliases run them under this host.

use crate::memory::tools::host::HostMemoryTools;

/// `memory_tools_list` over this host.
pub type MemoryToolsListTool = tinymemory_tools::tool_memory::MemoryToolsListTool<HostMemoryTools>;
/// `memory_tools_put` over this host.
pub type MemoryToolsPutTool = tinymemory_tools::tool_memory::MemoryToolsPutTool<HostMemoryTools>;

#[cfg(test)]
#[path = "guard_tests.rs"]
mod tests;
