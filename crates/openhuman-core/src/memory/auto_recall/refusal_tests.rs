use super::*;

#[test]
fn a_hosted_credit_refusal_is_out_of_credits() {
    let error = MemoryError::BudgetExceeded(
        "[USER_INSUFFICIENT_CREDITS] memory API memory/recall on api.example (HTTP 402 \
         Payment Required): Insufficient credits — insufficient credits"
            .into(),
    );
    assert_eq!(MemoryRefusal::of(&error), Some(MemoryRefusal::OutOfCredits));
}

#[test]
fn a_401_is_a_session_the_engine_did_not_accept() {
    let error = MemoryError::Unauthorized(
        "[UNAUTHORIZED] memory API memory/recall on api.example (HTTP 401 Unauthorized): \
         Invalid token — the session expired or the API key was rejected; re-authenticate"
            .into(),
    );
    assert_eq!(
        MemoryRefusal::of(&error),
        Some(MemoryRefusal::SessionExpired)
    );
}

#[test]
fn a_403_is_a_refused_credential_not_a_lapsed_session() {
    let error = MemoryError::Unauthorized(
        "[FORBIDDEN] memory API memory/recall on api.example (HTTP 403 Forbidden): API key \
         is missing the required scope: memory — the session expired or the API key was \
         rejected; re-authenticate"
            .into(),
    );
    assert_eq!(MemoryRefusal::of(&error), Some(MemoryRefusal::Forbidden));
}

#[test]
fn an_outage_is_unavailable() {
    for error in [
        MemoryError::Unavailable("[RATE_LIMITED] memory API memory/recall (HTTP 429)".into()),
        MemoryError::Unreachable("memory API request to api.example: could not connect".into()),
        MemoryError::Timeout("memory API request to api.example: timed out".into()),
    ] {
        assert_eq!(
            MemoryRefusal::of(&error),
            Some(MemoryRefusal::Unavailable),
            "{error}"
        );
    }
}

#[test]
fn an_ordinary_failure_is_not_a_refusal() {
    for error in [
        MemoryError::Invalid("namespace must not be empty".into()),
        MemoryError::Backend("engine answered 418".into()),
        MemoryError::BudgetExceeded("the answer exceeded its token budget".into()),
    ] {
        assert_eq!(MemoryRefusal::of(&error), None, "{error}");
    }
}

#[test]
fn the_refusal_block_heads_the_reason_with_the_usual_banner() {
    let block = render_refusal_block(MemoryRefusal::OutOfCredits);
    assert!(block.starts_with(AUTO_RECALL_BANNER), "{block}");
    assert!(block.contains("out of credits"), "{block}");
    assert!(
        block.contains("rather than that it was never stored"),
        "{block}"
    );
}
