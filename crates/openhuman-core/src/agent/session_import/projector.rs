//! Host projection of a durable transcript row into a session-store journal
//! record.
//!
//! The importer, the live dual-write and the shadow read (all in
//! `tinyagents_session::transcript::import`) take this as their
//! `JournalProjector`. It goes through the host's `ChatMessage` shape so the
//! journal carries the reconstructed sidecar metadata (`openhuman_turn_usage`,
//! tool-failure and replay markers) exactly as a transcript read-back does.

use tinyagents_session::transcript::import::types::JournalMessage;
use tinyagents_session::transcript::TranscriptMessage;

use crate::agent::messages::{chat_message_from_transcript, ChatMessage};

impl From<&ChatMessage> for JournalMessage {
    fn from(msg: &ChatMessage) -> Self {
        Self {
            id: msg.id.clone(),
            role: msg.role.clone(),
            content: msg.content.clone(),
            extra_metadata: msg.extra_metadata.clone(),
        }
    }
}

/// The `JournalProjector` OpenHuman passes to the importer.
pub fn journal_message_from_transcript(message: TranscriptMessage) -> JournalMessage {
    JournalMessage::from(&chat_message_from_transcript(message))
}
