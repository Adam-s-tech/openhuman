//! Why a lookup could not run, when the engine said why.
//!
//! A remote engine can refuse a recall for reasons that are not "nothing
//! relevant": the hosted account is out of credits, the session was not
//! accepted, the credential lacks the memory scope, or the service cannot be
//! reached. The lane used to log those and carry on as though memory were
//! empty, so the model told the user a fact they had saved was never stored.
//! A refused lookup now puts that reason in front of the model instead, in the
//! block's usual place, so the answer can say what is actually wrong.

use crate::memory::api::error::MemoryError;
use crate::memory::ops::engine::{
    classify_memory_error, INSUFFICIENT_CREDITS_PREFIX, MEMORY_FORBIDDEN_PREFIX,
    MEMORY_UNREACHABLE_PREFIX, SESSION_EXPIRED_PREFIX,
};

use super::{fits_within, AUTO_RECALL_BANNER};

/// Why memory could not be searched for a message.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemoryRefusal {
    /// The hosted engine's account has no credits left.
    OutOfCredits,
    /// The engine did not accept the session.
    SessionExpired,
    /// The engine refused the credential outright (HTTP 403).
    Forbidden,
    /// The engine could not be reached, or answered that it cannot serve now.
    Unavailable,
}

impl MemoryRefusal {
    /// The refusal `error` stands for, or `None` for an ordinary failure.
    pub fn of(error: &MemoryError) -> Option<Self> {
        let classified = classify_memory_error(error);
        [
            (INSUFFICIENT_CREDITS_PREFIX, Self::OutOfCredits),
            (SESSION_EXPIRED_PREFIX, Self::SessionExpired),
            (MEMORY_FORBIDDEN_PREFIX, Self::Forbidden),
            (MEMORY_UNREACHABLE_PREFIX, Self::Unavailable),
        ]
        .into_iter()
        .find_map(|(prefix, refusal)| classified.starts_with(prefix).then_some(refusal))
    }

    /// A stable label for log lines.
    pub fn label(self) -> &'static str {
        match self {
            Self::OutOfCredits => "out_of_credits",
            Self::SessionExpired => "session_expired",
            Self::Forbidden => "forbidden",
            Self::Unavailable => "unavailable",
        }
    }

    /// What the model is told, as the clause after "could not be searched".
    fn reason(self) -> &'static str {
        match self {
            Self::OutOfCredits => {
                "the OpenHuman account is out of credits, and hosted memory is billed in \
                 credits; it works again once credits are added"
            }
            Self::SessionExpired => {
                "the memory service did not accept the session; it works again after the \
                 user signs in"
            }
            Self::Forbidden => {
                "the memory service refused this account's credential; an API key needs \
                 the memory scope"
            }
            Self::Unavailable => {
                "the memory service could not be reached; it may work again in a moment"
            }
        }
    }
}

/// The block for a lookup that was refused and found nothing else: the usual
/// banner over one line saying why memory is out of reach.
///
/// It is held to `recall_max_chars` like any recall block. The reason is one
/// sentence, so a cap it does not fit whole leaves the block out rather than
/// cutting it.
pub(crate) fn render_refusal_block(
    refusal: MemoryRefusal,
    recall_max_chars: Option<usize>,
) -> Option<String> {
    let block = format!(
        "{AUTO_RECALL_BANNER}\n\nMemory could not be searched for this message: {}. \
         If the user asks about something they saved, say that memory is unavailable \
         and why, rather than that it was never stored.\n\n",
        refusal.reason()
    );
    let chars = block.chars().count();
    if !fits_within(0, chars, recall_max_chars) {
        log::debug!(
            "[auto_recall] refusal block omitted: {chars} chars would exceed \
             recall_max_chars={recall_max_chars:?} refusal={}",
            refusal.label()
        );
        return None;
    }
    Some(block)
}

#[cfg(test)]
#[path = "refusal_tests.rs"]
mod tests;
