use super::*;
use crate::web_chat::web_errors::is_turn_timeout_error;

fn hosted(kind: HostedErrorKind, bound: Option<TimeoutBound>) -> HostedError {
    HostedError {
        kind,
        message: "hosted agent invocation timed out".to_string(),
        timeout_bound: bound,
        run: None,
    }
}

#[test]
fn a_per_call_timeout_names_the_per_model_call_bound() {
    let err = run_error_from_hosted(hosted(
        HostedErrorKind::Timeout,
        Some(TimeoutBound::PerModelCall),
    ));
    assert!(matches!(err, TinyAgentsError::Timeout(_)));
    let text = err.to_string();
    assert_eq!(
        crate::web_chat::timeout_bound_tag(&text),
        "per_model_call",
        "{text}"
    );
    assert!(is_turn_timeout_error(&text), "user-facing class unchanged");
}

#[test]
fn a_run_budget_timeout_names_the_run_bound() {
    let text = run_error_from_hosted(hosted(HostedErrorKind::Timeout, Some(TimeoutBound::Run)))
        .to_string();
    assert_eq!(crate::web_chat::timeout_bound_tag(&text), "run_remaining");
    assert!(is_turn_timeout_error(&text));
}

#[test]
fn a_non_timeout_failure_is_converted_as_before() {
    let err = run_error_from_hosted(hosted(HostedErrorKind::Provider, None));
    assert!(matches!(err, TinyAgentsError::Model(_)));
    assert_eq!(crate::web_chat::timeout_bound_tag(&err.to_string()), "none");
}
