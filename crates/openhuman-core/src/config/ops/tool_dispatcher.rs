//! Validation for the `[agent] tool_dispatcher` setting.

/// Accepted spellings of `[agent] tool_dispatcher`. `auto` is the default:
/// native structured calls when the provider supports them, else JSON-in-tag.
pub const TOOL_DISPATCHER_CHOICES: [&str; 6] =
    ["auto", "native", "xml", "pformat", "python", "typescript"];

/// Trim and lowercase `raw`, rejecting anything outside
/// [`TOOL_DISPATCHER_CHOICES`].
pub fn normalize_tool_dispatcher(raw: &str) -> Result<String, String> {
    let normalized = raw.trim().to_ascii_lowercase();
    if !TOOL_DISPATCHER_CHOICES.contains(&normalized.as_str()) {
        log::warn!("[config][agent] rejected tool_dispatcher={normalized:?}");
        return Err(format!(
            "invalid tool_dispatcher '{normalized}' (expected {})",
            TOOL_DISPATCHER_CHOICES.join(" | ")
        ));
    }
    Ok(normalized)
}
