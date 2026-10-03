use super::*;
use tinyagents_harness::summarization::DEFAULT_SUMMARIZE_KEEP_LAST;

#[test]
fn no_window_and_no_override_installs_no_compression() {
    assert!(compression_policy(None, None).is_none());
    assert!(compression_policy(Some(0), None).is_none());
    assert!(compression_policy(None, Some(0)).is_none());
}

#[test]
fn a_known_window_triggers_at_ninety_percent() {
    let policy = compression_policy(Some(200_000), None).expect("policy");
    assert_eq!(policy.trigger_budget(), 180_000);
    assert_eq!(policy.keep_last, DEFAULT_SUMMARIZE_KEEP_LAST);
}

#[test]
fn the_override_replaces_the_window_fraction() {
    let policy = compression_policy(Some(1_000_000), Some(64_000)).expect("policy");
    assert_eq!(policy.trigger_budget(), 64_000);
    assert!(policy.exceeds_trigger(64_001));
    assert!(!policy.exceeds_trigger(63_000));
    assert_eq!(policy.keep_last, DEFAULT_SUMMARIZE_KEEP_LAST);
}

#[test]
fn the_override_works_without_a_known_window() {
    let policy = compression_policy(None, Some(64_000)).expect("policy");
    assert_eq!(policy.trigger_budget(), 64_000);
    assert_eq!(policy.keep_last, DEFAULT_SUMMARIZE_KEEP_LAST);
}
