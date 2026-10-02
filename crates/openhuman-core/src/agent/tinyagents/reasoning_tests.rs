use super::*;

#[test]
fn parses_wire_tokens_and_aliases() {
    assert_eq!(parse_reasoning_effort("high"), Some(ReasoningEffort::High));
    assert_eq!(parse_reasoning_effort(" Low "), Some(ReasoningEffort::Low));
    assert_eq!(parse_reasoning_effort("off"), Some(ReasoningEffort::None));
    assert_eq!(parse_reasoning_effort("max"), Some(ReasoningEffort::XHigh));
    assert_eq!(parse_reasoning_effort("xhigh"), Some(ReasoningEffort::XHigh));
    assert_eq!(parse_reasoning_effort("turbo"), None);
}

#[test]
fn unset_config_leaves_the_provider_default() {
    assert_eq!(reasoning_for_config(&Config::default()), None);
}

#[test]
fn configured_effort_becomes_the_turn_reasoning() {
    let mut config = Config::default();
    config.runtime.reasoning_effort = Some("high".into());
    assert_eq!(
        reasoning_for_config(&config),
        Some(ReasoningConfig::effort(ReasoningEffort::High))
    );
}

#[test]
fn reasoning_disabled_without_an_effort_asks_for_none() {
    let mut config = Config::default();
    config.runtime.reasoning_enabled = Some(false);
    assert_eq!(
        reasoning_for_config(&config),
        Some(ReasoningConfig::effort(ReasoningEffort::None))
    );
}

#[test]
fn explicit_effort_wins_over_reasoning_disabled() {
    let mut config = Config::default();
    config.runtime.reasoning_enabled = Some(false);
    config.runtime.reasoning_effort = Some("low".into());
    assert_eq!(
        reasoning_for_config(&config),
        Some(ReasoningConfig::effort(ReasoningEffort::Low))
    );
}

#[test]
fn unknown_effort_falls_back_to_the_provider_default() {
    let mut config = Config::default();
    config.runtime.reasoning_effort = Some("turbo".into());
    assert_eq!(reasoning_for_config(&config), None);
}

#[test]
fn thread_choice_wins_over_config_and_clears_back_to_it() {
    let thread = "reasoning-test-thread-precedence";
    let mut config = Config::default();
    config.runtime.reasoning_effort = Some("low".into());

    apply_requested_effort(thread, Some("high")).expect("valid effort");
    assert_eq!(
        turn_reasoning(Some(thread), Some(&config)),
        Some(ReasoningConfig::effort(ReasoningEffort::High))
    );

    // Absent leaves the thread's choice untouched.
    apply_requested_effort(thread, None).expect("absent is fine");
    assert_eq!(thread_effort(thread), Some(ReasoningEffort::High));

    apply_requested_effort(thread, Some("default")).expect("clear");
    assert_eq!(thread_effort(thread), None);
    assert_eq!(
        turn_reasoning(Some(thread), Some(&config)),
        Some(ReasoningConfig::effort(ReasoningEffort::Low))
    );
}

#[test]
fn unknown_requested_effort_is_rejected_and_keeps_the_prior_choice() {
    let thread = "reasoning-test-thread-reject";
    apply_requested_effort(thread, Some("off")).expect("valid effort");
    assert!(apply_requested_effort(thread, Some("turbo")).is_err());
    assert_eq!(thread_effort(thread), Some(ReasoningEffort::None));
    set_thread_effort(thread, None);
}

#[test]
fn turn_without_thread_or_config_has_no_reasoning() {
    assert_eq!(turn_reasoning(None, None), None);
}
