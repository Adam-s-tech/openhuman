use super::*;
use crate::memory::api::types::{MemoryCategory, MemoryEntry};
use crate::memory::guard::in_memory::guarded_fixed_recall;

// The lane against remote engines (openhuman#6718): an engine that ranks its
// recall without scoring it, and an engine that refuses the lookup. Fixtures
// (`note`, `Scripted`, the constants) live in the parent module.

/// A note from an engine that does not score: its similarity reads 0.0.
fn unscored(key: &str, content: &str) -> NamespaceMemoryHit {
    note(key, content, 0.0)
}

fn out_of_credits() -> MemoryError {
    MemoryError::BudgetExceeded(
        "[USER_INSUFFICIENT_CREDITS] memory API memory/recall on api.example (HTTP 402 \
         Payment Required): Insufficient credits — insufficient credits"
            .into(),
    )
}

fn unreachable() -> MemoryError {
    MemoryError::Unreachable("memory API request to api.example: could not connect".into())
}

#[test]
fn select_unscored_notes_keeps_the_engines_first_few_with_a_body() {
    let kept = select_unscored_notes(vec![
        unscored("a", "first"),
        unscored("empty", "   "),
        unscored("b", "second"),
        unscored("c", "third"),
        unscored("d", "fourth"),
    ]);
    let keys: Vec<&str> = kept.iter().map(|n| n.key.as_str()).collect();
    assert_eq!(keys, ["a", "b", "c"]);
    assert_eq!(kept.len(), AUTO_RECALL_UNSCORED_NOTES);
}

#[tokio::test]
async fn an_unscored_engines_notes_reach_the_block_in_its_order() {
    // Every similarity reads 0.0, so the scored floor would drop them all.
    let source = Scripted::hits(vec![]).with_unscored_notes(vec![
        unscored("favourite_tea_oolong", TEA_NOTE),
        unscored("rent", "Rent is due on the 5th."),
        unscored("router", "The router is in the hallway cupboard."),
        unscored("cat", "The cat eats salmon kibble."),
    ]);
    let lane = AutoRecall::new(source, true, None);
    let block = lane.block_for(TEA_QUESTION).await.expect("a block");
    assert!(block.contains(TEA_NOTE), "{block}");
    assert!(
        block.contains("hallway cupboard"),
        "the first three: {block}"
    );
    assert!(!block.contains("salmon kibble"), "past the cap: {block}");
}

#[tokio::test]
async fn a_scored_engines_zero_similarity_is_still_floored() {
    let source =
        Scripted::hits(vec![]).with_notes(vec![note("favourite_tea_oolong", TEA_NOTE, 0.0)]);
    let lane = AutoRecall::new(source, true, None);
    assert!(lane.block_for(TEA_QUESTION).await.is_none());
}

#[tokio::test]
async fn a_credit_refusal_tells_the_model_why_memory_is_empty() {
    let source = Scripted::hits(vec![]).with_refused_notes(out_of_credits);
    let lane = AutoRecall::new(source, true, None);
    let block = lane.block_for(TEA_QUESTION).await.expect("a refusal block");
    assert!(block.starts_with(AUTO_RECALL_BANNER), "{block}");
    assert!(block.contains("out of credits"), "{block}");
}

#[tokio::test]
async fn a_refusal_block_stays_within_the_guards_recall_budget() {
    let tight = AutoRecall::new(
        Scripted::hits(vec![]).with_refused_notes(out_of_credits),
        true,
        Some(40),
    );
    assert!(tight.block_for(TEA_QUESTION).await.is_none());
    let roomy = AutoRecall::new(
        Scripted::hits(vec![]).with_refused_notes(out_of_credits),
        true,
        Some(4_000),
    );
    let block = roomy
        .block_for(TEA_QUESTION)
        .await
        .expect("a refusal block");
    assert!(block.contains("out of credits"), "{block}");
}

#[tokio::test]
async fn an_unreachable_engine_is_named_not_mistaken_for_an_empty_one() {
    let source = Scripted::hits(vec![]).with_refused_notes(unreachable);
    let lane = AutoRecall::new(source, true, None);
    let block = lane.block_for(TEA_QUESTION).await.expect("a refusal block");
    assert!(block.contains("could not be reached"), "{block}");
}

#[tokio::test]
async fn a_refusal_does_not_hide_what_the_other_leg_found() {
    let source =
        Scripted::hits(vec![hit("Idol: Virat Kohli", 0.9)]).with_refused_notes(out_of_credits);
    let lane = AutoRecall::new(source, true, None);
    let block = lane.block_for(QUESTION).await.expect("a block");
    assert!(block.contains("Virat Kohli"), "{block}");
}

#[tokio::test]
async fn an_ordinary_notes_failure_is_still_no_block() {
    let source = Scripted::hits(vec![]).with_failing_notes("store locked");
    let lane = AutoRecall::new(source, true, None);
    assert!(lane.block_for(TEA_QUESTION).await.is_none());
}

/// An entry as a remote engine returns it from ranked recall: no score.
fn remote_entry(key: &str, content: &str) -> MemoryEntry {
    MemoryEntry {
        id: key.to_string(),
        key: key.to_string(),
        content: content.to_string(),
        namespace: Some(AUTO_RECALL_NOTES_NAMESPACE.to_string()),
        category: MemoryCategory::Core,
        timestamp: String::new(),
        session_id: None,
        score: None,
        taint: MemoryTaint::Internal,
    }
}

#[tokio::test]
async fn from_guard_keeps_an_unscored_engines_notes() {
    // No retrieval family: the notes come from the mandatory ranked recall,
    // whose entries carry no score (hosted CortexDB).
    let guard = guarded_fixed_recall(vec![
        remote_entry("favourite_tea_oolong", TEA_NOTE),
        remote_entry("rent", "Rent is due on the 5th."),
    ]);
    let lane = AutoRecall::from_guard(guard);
    assert!(lane.enabled());
    let block = lane.block_for(TEA_QUESTION).await.expect("a block");
    assert!(block.contains(TEA_NOTE), "{block}");
}
