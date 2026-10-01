use super::*;
use crate::memory::api::provider::SegmentStatus;

fn segment() -> ConversationSegment {
    ConversationSegment {
        segment_id: "segment".into(),
        session_id: "session".into(),
        namespace: "global".into(),
        start_episodic_id: 20,
        end_episodic_id: Some(24),
        start_timestamp: 100.000_9,
        end_timestamp: Some(100.001_1),
        turn_count: 3,
        summary: None,
        embedding: None,
        open: false,
        // #6186: the lifecycle marker the contract now carries. `Closed`
        // is what a segment whose recap failed is left as.
        status: Some(SegmentStatus::Closed),
        start_seq: Some(10),
        end_seq: Some(14),
    }
}

fn entry(sequence: Option<u32>, id: Option<i64>, timestamp: f64) -> SessionEntry {
    SessionEntry {
        sequence,
        turn: EpisodicTurn {
            id,
            session_id: "session".into(),
            timestamp,
            role: "user".into(),
            content: "content".into(),
            lesson: None,
            tool_calls_json: None,
            cost_microdollars: 0,
        },
    }
}

#[test]
fn segment_membership_uses_sequence_instead_of_rounded_timestamp() {
    let segment = segment();

    assert!(!entry(Some(9), None, 100.001).is_in_segment(&segment));
    assert!(entry(Some(10), None, 100.000).is_in_segment(&segment));
    assert!(entry(Some(15), None, 101.0).is_in_segment(&segment));
    assert!(!entry(Some(16), None, 100.001).is_in_segment(&segment));
}

#[test]
fn segment_membership_falls_back_to_episodic_id() {
    let mut segment = segment();
    segment.start_seq = None;
    segment.end_seq = None;

    assert!(!entry(None, Some(19), 100.001).is_in_segment(&segment));
    assert!(entry(None, Some(20), 100.000).is_in_segment(&segment));
    assert!(entry(None, Some(25), 101.0).is_in_segment(&segment));
    assert!(!entry(None, Some(26), 100.001).is_in_segment(&segment));
}

/// The recap deadline clears two of `tinymemory`'s retry attempts.
///
/// Not a behavioural test — an arithmetic one, and it exists because the two
/// numbers live in different repositories and nothing else would catch them
/// drifting apart. `tinymemory`'s retry stops starting further attempts once
/// ~20 s have elapsed, so its realistic worst case is two folds back to back.
/// A `RECAP_DEADLINE` under that would cut the chain before the second attempt
/// and silently turn the retry into dead code — the failure this pins.
#[test]
fn the_recap_deadline_leaves_room_for_the_drivers_retry() {
    const TINYMEMORY_RETRY_CEILING: Duration = Duration::from_secs(20);

    assert!(
        RECAP_DEADLINE > TINYMEMORY_RETRY_CEILING * 2,
        "RECAP_DEADLINE ({RECAP_DEADLINE:?}) must clear two attempts of the \
         driver's {TINYMEMORY_RETRY_CEILING:?} retry window, or the retry can \
         never fire a second attempt"
    );
    assert!(
        RECAP_DEADLINE < Duration::from_secs(600),
        "RECAP_DEADLINE must stay well under tinyinference's 600s request \
         default, which is the bound it exists to replace"
    );
}

// ── Host-side fold, for a driver with no model ──────────────────────────────

fn turn(role: &str, content: &str) -> EpisodicTurn {
    EpisodicTurn {
        id: None,
        session_id: "s1".into(),
        timestamp: 1.0,
        role: role.into(),
        content: content.into(),
        lesson: None,
        tool_calls_json: None,
        cost_microdollars: 0,
    }
}

/// The transcript keeps the newest turns that fit, in order, and skips blank
/// ones.
#[test]
fn a_transcript_keeps_the_newest_turns_that_fit() {
    let turns = [
        turn("user", "first question"),
        turn("assistant", "   "),
        turn("user", "second question"),
        turn("assistant", "the answer"),
    ];
    let refs: Vec<&EpisodicTurn> = turns.iter().collect();
    assert_eq!(
        transcript(&refs, 10_000),
        "user: first question\n\nuser: second question\n\nassistant: the answer"
    );
    // Two paragraphs of 21 characters, each with its separator, need 46.
    let tight = transcript(&refs, 46);
    assert_eq!(tight, "user: second question\n\nassistant: the answer");
    assert_eq!(
        transcript(&refs[..1], 5),
        "user:",
        "a turn longer than the whole budget is cut to it, not sent whole"
    );
}

/// A cut never splits a character, however the budget falls.
#[test]
fn an_oversized_turn_is_cut_on_a_character_boundary() {
    let turns = [turn("user", "café ☕ au lait")];
    let refs: Vec<&EpisodicTurn> = turns.iter().collect();
    for max in 0..40 {
        let cut = transcript(&refs, max);
        assert!(cut.len() <= max, "max={max} gave {} bytes", cut.len());
        assert!(
            "user: café ☕ au lait".starts_with(&cut),
            "max={max}: {cut}"
        );
    }
}

struct ScriptedRecap;

#[async_trait::async_trait]
impl tinyinference_llm::model::ChatModel<()> for ScriptedRecap {
    async fn invoke(
        &self,
        _state: &(),
        request: tinyinference_llm::model::ModelRequest,
    ) -> tinyinference_llm::Result<tinyinference_llm::model::ModelResponse> {
        let seen = format!("{request:?}");
        assert!(seen.contains("user: book the flights"), "{seen}");
        Ok(tinyinference_llm::model::ModelResponse::assistant(
            "  They booked the flights.  ",
        ))
    }
}

/// When the driver has no model, the host's own folds the segment, and the
/// recap comes back trimmed.
#[tokio::test]
async fn the_host_model_folds_a_segment_the_driver_cannot() {
    let _model = crate::inference::provider::factory::test_provider_override::install_model(
        std::sync::Arc::new(ScriptedRecap),
    );
    let config = crate::config::Config::default();
    let turns = [turn("user", "book the flights"), turn("assistant", "done")];
    let refs: Vec<&EpisodicTurn> = turns.iter().collect();
    let recap = fold_with_host_model(&config, &refs)
        .await
        .expect("the host folds");
    assert_eq!(recap, "They booked the flights.");
    let none = fold_with_host_model(&config, &[]).await.expect("no turns");
    assert!(none.is_empty(), "no turns, no model call, no recap");
}
