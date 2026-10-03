use super::*;

/// Issue #6960: a compaction that fires mid-turn must keep the turn's
/// assignment verbatim and size its tail in tokens, not by message count.
#[test]
fn compression_policy_keeps_a_token_tail_and_pins_the_turn_user_message() {
    let policy = compression_policy(200_000);

    assert_eq!(policy.context_window, Some(200_000));
    assert_eq!(
        policy.threshold_fraction,
        tinyagents_harness::summarization::DEFAULT_SUMMARIZE_THRESHOLD_FRACTION
    );
    assert_eq!(policy.keep_recent_tokens, Some(60_000));
    assert!(policy.pin_turn_user_message);
}
