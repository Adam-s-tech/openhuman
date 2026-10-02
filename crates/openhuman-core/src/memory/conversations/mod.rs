//! Workspace-backed conversation thread/message storage — the whole thing,
//! store and wiring both.
//!
//! Conversations are JSONL files under `<workspace>/memory/conversations/`:
//! thread metadata as an append-only upsert/delete log in `threads.jsonl`, and
//! each thread's messages in a dedicated file under
//! `threads/<hex(thread_id)>.jsonl`. This is **transcript persistence** — the
//! raw records plus a trigram/CJK-bigram index for cross-thread substring
//! search over them. The summary-tree archival of the same transcripts is
//! [`crate::memory::tree`], a different index answering a different
//! question.
//!
//! The store itself (on-disk format, locking, warm index cache, CRUD and
//! search) is the `tinymemory-conversations` crate, re-exported below so
//! callers keep naming `crate::memory::conversations::{...}`. What stays here
//! is host wiring:
//!
//! - [`blocking`] - `spawn_blocking` wrappers. Every store entry point is
//!   synchronous and can take `parking_lot` locks across fsync'd file IO, so an
//!   `async fn` that calls one directly parks a tokio **worker** thread for the
//!   whole wait. Request paths must use these (#5156).
//! - `bus` - the `core::bus` subscriber that mirrors inbound and processed
//!   channel turns into the store, so Slack/Telegram/... persist alongside the
//!   UI's own threads.
//!
//! The on-disk format is unchanged: the root is still
//! `<workspace>/memory/conversations`.

pub mod blocking;

mod bus;
use tinymemory_conversations as store;

pub use bus::register_conversation_persistence_subscriber;
pub use store::{
    append_message, delete_messages_from, delete_thread, ensure_thread, get_messages,
    is_deterministic_message_id, list_threads, purge_threads, reply_run_id, run_reply_message_id,
    update_message, update_thread_labels, update_thread_title, ConversationMessage,
    ConversationMessagePatch, ConversationPurgeStats, ConversationStore, ConversationThread,
    CreateConversationThread, CrossThreadHit,
};
