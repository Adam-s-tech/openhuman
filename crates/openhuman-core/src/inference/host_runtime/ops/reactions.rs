//! The emoji reaction decision: asking the local model whether to react and
//! extracting the emoji it picked.

use crate::config::Config;
use crate::core::Outcome;
use crate::inference::host_runtime as local_ai;

/// Result of the reaction-decision prompt.
#[derive(Debug, serde::Serialize)]
pub struct ReactionDecision {
    /// Whether the model thinks a reaction is appropriate.
    pub should_react: bool,
    /// The emoji to use (only meaningful when `should_react` is true).
    pub emoji: Option<String>,
}

/// Evaluates whether the assistant should add an emoji reaction to a user message.
///
/// This uses the local model to make a quick decision based on the message
/// content and the channel context.
pub async fn local_ai_should_react(
    config: &Config,
    message: &str,
    channel_type: &str,
) -> Result<Outcome<ReactionDecision>, String> {
    tracing::debug!(
        channel_type,
        msg_len = message.len(),
        "[local_ai:should_react] evaluating reaction"
    );

    if message.trim().is_empty() {
        return Ok(Outcome::single_log(
            ReactionDecision {
                should_react: false,
                emoji: None,
            },
            "empty message — no reaction",
        ));
    }

    let service = local_ai::global(config);
    let status = service.status();
    if !matches!(status.state.as_str(), "ready") {
        tracing::debug!("[local_ai:should_react] local model not ready, skipping");
        return Ok(Outcome::single_log(
            ReactionDecision {
                should_react: false,
                emoji: None,
            },
            "local model not ready",
        ));
    }

    let prompt = format!(
        "You decide whether an AI assistant should react to a user message with a single emoji. \
         Consider the channel context: casual channels (discord, telegram) get more frequent \
         reactions with playful emojis, while professional channels (web, slack, email) are more \
         reserved — only react to clearly emotional or noteworthy messages.\n\n\
         Channel: {channel_type}\nUser message: {message}\n\n\
         Reply with EXACTLY one word: either NONE (no reaction) or a single emoji character."
    );

    let runtime = crate::inference::local_runtime_config(config);
    let Some(_permit) = crate::cron::scheduler_gate::wait_for_capacity().await else {
        return Ok(Outcome::single_log(
            ReactionDecision {
                should_react: false,
                emoji: None,
            },
            "local inference paused while signed out",
        ));
    };
    let output = service.prompt(&runtime, &prompt, Some(8), true).await;

    let decision = match output {
        Ok(raw) => {
            let trimmed = raw.trim();
            tracing::debug!(
                output_len = trimmed.len(),
                "[local_ai:should_react] model response"
            );
            if trimmed.eq_ignore_ascii_case("NONE") || trimmed.is_empty() {
                ReactionDecision {
                    should_react: false,
                    emoji: None,
                }
            } else {
                // Extract the first emoji-like character(s) from the response
                let emoji = tinyinference_llm::classification::extract_first_emoji(trimmed);
                match emoji {
                    Some(e) => ReactionDecision {
                        should_react: true,
                        emoji: Some(e),
                    },
                    None => ReactionDecision {
                        should_react: false,
                        emoji: None,
                    },
                }
            }
        }
        Err(e) => {
            tracing::debug!(error = %e, "[local_ai:should_react] inference failed, skipping");
            ReactionDecision {
                should_react: false,
                emoji: None,
            }
        }
    };

    tracing::debug!(
        should_react = decision.should_react,
        emoji = ?decision.emoji,
        "[local_ai:should_react] decision"
    );
    Ok(Outcome::single_log(decision, "reaction decision completed"))
}
