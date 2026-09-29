use super::*;

#[test]
fn dropped_attempt_releases_in_flight_state() {
    let state = RetryState::default();
    let generation = begin_outage_attempt(Some(&state), "stub-cloud").expect("attempt");
    drop(OutageAttempt {
        state: &state,
        key: "stub-cloud",
        generation,
    });

    assert!(!state.lock().expect("retry state")["stub-cloud"].in_flight);
}
