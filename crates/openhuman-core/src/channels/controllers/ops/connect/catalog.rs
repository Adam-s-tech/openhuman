//! Read-only channel catalog lookups.

use crate::core::Outcome;

use super::super::super::definitions::{
    all_channel_definitions, find_channel_definition, ChannelDefinition,
};

/// List all available channel definitions.
pub async fn list_channels() -> Result<Outcome<Vec<ChannelDefinition>>, String> {
    Ok(Outcome::new(all_channel_definitions(), vec![]))
}

/// Describe a single channel by id.
pub async fn describe_channel(channel_id: &str) -> Result<Outcome<ChannelDefinition>, String> {
    let def = find_channel_definition(channel_id)
        .ok_or_else(|| format!("unknown channel: {channel_id}"))?;
    Ok(Outcome::new(def, vec![]))
}
