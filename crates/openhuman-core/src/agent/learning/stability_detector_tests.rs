use super::*;
use crate::agent::learning::candidate::{Buffer, EvidenceRef, FacetClass, LearningCandidate};

fn make_detector() -> StabilityDetector {
    let cache = crate::agent::learning::test_profile::in_memory_cache();
    // Use a private buffer so tests don't interfere with the global singleton.
    let buffer: &'static Buffer = Box::leak(Box::new(Buffer::new(256)));
    StabilityDetector {
        cache,
        buffer,
        last_rebuild_at: std::sync::Mutex::new(None),
        rewrite_tolerance: REWRITE_TOLERANCE,
        state_path: None,
        rebuild_turn: std::sync::Arc::default(),
    }
}

fn make_candidate(
    class: FacetClass,
    key: &str,
    value: &str,
    cue: CueFamily,
    observed_at: f64,
) -> LearningCandidate {
    LearningCandidate {
        class,
        key: key.into(),
        value: value.into(),
        cue_family: cue,
        evidence: EvidenceRef::Episodic { episodic_id: 1 },
        initial_confidence: 0.8,
        observed_at,
    }
}

// ── stability formula ────────────────────────────────────────────────────

#[test]
fn stability_pinned_returns_infinity() {
    let s = stability(
        CueFamily::Behavioral,
        5,
        0.0,
        1000.0,
        FacetClass::Style,
        false,
        UserState::Pinned,
    );
    assert!(s.is_infinite() && s > 0.0);
}

#[test]
fn stability_forgotten_returns_zero() {
    let s = stability(
        CueFamily::Explicit,
        100,
        0.0,
        1000.0,
        FacetClass::Style,
        true,
        UserState::Forgotten,
    );
    assert_eq!(s, 0.0);
}

#[test]
fn stability_explicit_doubles_score() {
    let base = stability(
        CueFamily::Explicit,
        3,
        1_000_000.0,
        1_000_001.0,
        FacetClass::Style,
        false, // no_explicit
        UserState::Auto,
    );
    let with_explicit = stability(
        CueFamily::Explicit,
        3,
        1_000_000.0,
        1_000_001.0,
        FacetClass::Style,
        true, // has_explicit
        UserState::Auto,
    );
    assert!(
        (with_explicit - 2.0 * base).abs() < 1e-9,
        "explicit multiplier must be exactly 2x: base={base:.6} explicit={with_explicit:.6}"
    );
}

#[test]
fn stability_decays_over_time() {
    let now = 1_000_000.0_f64;
    let recent = stability(
        CueFamily::Behavioral,
        5,
        now - 100.0, // observed 100 s ago
        now,
        FacetClass::Style,
        false,
        UserState::Auto,
    );
    let old = stability(
        CueFamily::Behavioral,
        5,
        now - HALF_LIFE_STYLE, // observed one half-life ago
        now,
        FacetClass::Style,
        false,
        UserState::Auto,
    );
    assert!(
        recent > old,
        "recent evidence should produce higher stability: recent={recent:.4} old={old:.4}"
    );
    // At exactly one half-life, recency = exp(-1) ≈ 0.368.
    assert!(
        old / recent < 0.4,
        "decay over one half-life should be substantial: ratio={}",
        old / recent
    );
}

// ── rebuild ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn rebuild_empty_buffer_no_candidates_is_noop() {
    let detector = make_detector();
    let now = 1_000_000.0;
    // No candidates, no existing rows → rebuild is a no-op.
    let outcome = detector.rebuild(now).await.unwrap();
    assert_eq!(outcome.added, 0);
    assert_eq!(outcome.evicted, 0);
    assert_eq!(outcome.kept, 0);
    assert_eq!(outcome.total_size, 0);
}

#[tokio::test]
async fn rebuild_strong_candidate_becomes_active() {
    let detector = make_detector();
    let now = 1_000_000.0;

    // Push enough explicit evidence to clear τ_promote.
    for i in 0..5 {
        detector.buffer.push(make_candidate(
            FacetClass::Style,
            "verbosity",
            "terse",
            CueFamily::Explicit,
            now - i as f64 * 10.0,
        ));
    }

    let outcome = detector.rebuild(now).await.unwrap();
    assert_eq!(outcome.added, 1);

    let actives = detector.cache.list_active().await.unwrap();
    assert_eq!(actives.len(), 1);
    assert_eq!(actives[0].key, "style/verbosity");
    assert_eq!(actives[0].value, "terse");
    assert_eq!(actives[0].state, FacetState::Active);
}

#[tokio::test]
async fn rebuild_conflict_resolution_picks_stronger_value() {
    let detector = make_detector();
    let now = 1_000_000.0;

    // 3 explicit candidates for "terse", 1 behavioral for "verbose".
    for _ in 0..3 {
        detector.buffer.push(make_candidate(
            FacetClass::Style,
            "verbosity",
            "terse",
            CueFamily::Explicit,
            now - 10.0,
        ));
    }
    detector.buffer.push(make_candidate(
        FacetClass::Style,
        "verbosity",
        "verbose",
        CueFamily::Behavioral,
        now - 5.0,
    ));

    detector.rebuild(now).await.unwrap();
    let actives = detector.cache.list_active().await.unwrap();
    assert!(!actives.is_empty(), "should have at least one active row");
    let verbosity = actives.iter().find(|f| f.key == "style/verbosity").unwrap();
    assert_eq!(
        verbosity.value, "terse",
        "terse had stronger evidence and should win"
    );
}

#[tokio::test]
async fn rebuild_class_budget_respected() {
    let detector = make_detector();
    let now = 1_000_000.0;

    // Push 6 different style keys — budget is BUDGET_STYLE = 4.
    for i in 0..6 {
        let key = format!("style_key_{i}");
        // Push several candidates per key so they clear τ_promote.
        for j in 0..5 {
            detector.buffer.push(LearningCandidate {
                class: FacetClass::Style,
                key: key.clone(),
                value: "v".into(),
                cue_family: CueFamily::Explicit,
                evidence: EvidenceRef::Episodic {
                    episodic_id: i * 10 + j,
                },
                initial_confidence: 0.9,
                observed_at: now - j as f64,
            });
        }
    }

    detector.rebuild(now).await.unwrap();

    let by_class = detector
        .cache
        .list_by_class(FacetClass::Style)
        .await
        .unwrap();
    assert!(
        by_class.len() <= BUDGET_STYLE,
        "style class should have at most {BUDGET_STYLE} active rows, got {}",
        by_class.len()
    );
}

#[tokio::test]
async fn rebuild_pinned_facet_stays_active_regardless_of_stability() {
    let detector = make_detector();
    let now = 1_000_000.0;

    // Manually insert a Pinned row.
    use tinymemory_api::provider::{FacetState, FacetType, UserState};
    let pinned = ProfileFacet {
        facet_id: "f-pinned".into(),
        facet_type: FacetType::Preference,
        key: "style/format".into(),
        value: "markdown".into(),
        confidence: 0.9,
        evidence_count: 1,
        source_segment_ids: None,
        first_seen_at: 1000.0,
        last_seen_at: 1000.0, // very old — would normally decay
        state: FacetState::Active,
        stability: 0.0,
        user_state: UserState::Pinned,
        evidence_refs: vec![],
        class: Some("style".into()),
        cue_families: None,
    };
    detector.cache.upsert(&pinned).await.unwrap();

    // No new candidates for this key → only decay applies.
    detector.rebuild(now).await.unwrap();

    let f = detector
        .cache
        .get("style/format")
        .await
        .unwrap()
        .expect("pinned row must survive");
    assert_eq!(f.state, FacetState::Active);
}

// ── half_life ────────────────────────────────────────────────────────────

#[test]
fn half_life_ordering_matches_spec() {
    // Identity decays slowest; Channel decays fastest.
    assert!(half_life(FacetClass::Identity) > half_life(FacetClass::Veto));
    assert!(half_life(FacetClass::Veto) > half_life(FacetClass::Tooling));
    assert!(half_life(FacetClass::Tooling) >= half_life(FacetClass::Goal));
    assert!(half_life(FacetClass::Goal) > half_life(FacetClass::Style));
    assert!(half_life(FacetClass::Style) > half_life(FacetClass::Channel));
}

// ── class_budget ────────────────────────────────────────────────────────

#[test]
fn class_budget_values_match_spec() {
    assert_eq!(class_budget(FacetClass::Style), 4);
    assert_eq!(class_budget(FacetClass::Identity), 4);
    assert_eq!(class_budget(FacetClass::Tooling), 5);
    assert_eq!(class_budget(FacetClass::Veto), 3);
    assert_eq!(class_budget(FacetClass::Goal), 3);
    assert_eq!(class_budget(FacetClass::Channel), 1);
}

// ── most_recent_reinforcement floor ───────────────────────────────────────

#[test]
fn reinforcement_floor_scopes_to_facet_class() {
    // With no candidates and no existing row, the result is purely the
    // class-scoped floor `now - half_life(class)`. This pins that the floor
    // tracks the facet's own class rather than a hardcoded one — the longer
    // half-lives (Goal, Identity) must floor further in the past than Style,
    // and Channel (shortest) closer to now.
    let now = 10_000_000.0;
    for class in [
        FacetClass::Identity,
        FacetClass::Veto,
        FacetClass::Tooling,
        FacetClass::Goal,
        FacetClass::Style,
        FacetClass::Channel,
    ] {
        let floor = most_recent_reinforcement(&[], None, None, now, class);
        assert_eq!(
            floor,
            now - half_life(class),
            "floor must use {class:?}'s own half-life"
        );
    }
    // Guard against a regression to a single hardcoded class: a class with a
    // different half-life than Style must produce a different floor.
    assert_ne!(
        most_recent_reinforcement(&[], None, None, now, FacetClass::Goal),
        most_recent_reinforcement(&[], None, None, now, FacetClass::Style),
    );
}

#[test]
fn a_row_counts_as_reinforced_at_the_previous_rebuild_at_the_latest() {
    // A row a rebuild skipped still carries the write time of an earlier
    // cycle; the previous rebuild would have refreshed it, so that is the
    // reinforcement stability reads. A later write time wins.
    let now = 10_000_000.0;
    let row = ProfileFacet {
        facet_id: "f".into(),
        facet_type: FacetType::Preference,
        key: "style/verbosity".into(),
        value: "terse".into(),
        confidence: 0.8,
        evidence_count: 5,
        source_segment_ids: None,
        first_seen_at: 1.0,
        last_seen_at: now - 7_200.0,
        state: FacetState::Active,
        stability: 1.6,
        user_state: UserState::Auto,
        evidence_refs: Vec::new(),
        class: Some("style".into()),
        cue_families: None,
    };
    let class = FacetClass::Style;
    assert_eq!(
        most_recent_reinforcement(&[], Some(&row), None, now, class),
        now - 7_200.0
    );
    assert_eq!(
        most_recent_reinforcement(&[], Some(&row), Some(now - 1_800.0), now, class),
        now - 1_800.0
    );
    assert_eq!(
        most_recent_reinforcement(&[], Some(&row), Some(now - 9_000.0), now, class),
        now - 7_200.0
    );
    assert_eq!(
        most_recent_reinforcement(&[], None, Some(now - 1_800.0), now, class),
        now - half_life(class),
        "a rebuild time says nothing about a row that did not exist"
    );
}

// ── merge_evidence_refs deduplication ─────────────────────────────────────

#[test]
fn merge_evidence_refs_removes_non_consecutive_duplicates() {
    // The bug this guards: a ref already in the existing row (Episodic 1)
    // that is re-emitted by a new candidate lands non-adjacent to its twin
    // once the two lists are concatenated ([1, 2, 1]). `Vec::dedup_by` only
    // collapses *consecutive* equals, so it would leave the duplicate in and
    // the refs list would grow every rebuild cycle. The set-based merge must
    // drop it, keeping the first occurrence and preserving order.
    let existing = vec![EvidenceRef::Episodic { episodic_id: 1 }];
    let new = vec![
        EvidenceRef::Episodic { episodic_id: 2 },
        EvidenceRef::Episodic { episodic_id: 1 },
    ];
    let merged = merge_evidence_refs(&existing, new);
    assert_eq!(
        merged,
        vec![
            EvidenceRef::Episodic { episodic_id: 1 },
            EvidenceRef::Episodic { episodic_id: 2 },
        ],
        "non-consecutive duplicate must be removed, first-seen order preserved"
    );
}

#[test]
fn merge_evidence_refs_dedups_within_a_single_cycle() {
    // Two candidates in the same cycle can reference the same evidence with
    // an unrelated ref between them; that also defeats consecutive-only dedup.
    let new = vec![
        EvidenceRef::TreeTopic {
            topic_id: "a".into(),
        },
        EvidenceRef::Episodic { episodic_id: 7 },
        EvidenceRef::TreeTopic {
            topic_id: "a".into(),
        },
    ];
    let merged = merge_evidence_refs(&[], new);
    assert_eq!(
        merged,
        vec![
            EvidenceRef::TreeTopic {
                topic_id: "a".into(),
            },
            EvidenceRef::Episodic { episodic_id: 7 },
        ],
    );
}

#[test]
fn merge_evidence_refs_is_idempotent_across_rebuilds() {
    // Re-running with the merged result as the new existing row and the same
    // candidates must not grow the list — the core invariant that the old
    // consecutive-only dedup violated.
    let existing = vec![
        EvidenceRef::Episodic { episodic_id: 1 },
        EvidenceRef::Episodic { episodic_id: 2 },
    ];
    let cands = vec![
        EvidenceRef::Episodic { episodic_id: 2 },
        EvidenceRef::Episodic { episodic_id: 1 },
    ];
    let first = merge_evidence_refs(&existing, cands.clone());
    let second = merge_evidence_refs(&first, cands);
    assert_eq!(first, second);
    assert_eq!(first.len(), 2);
}

// ── rewrite tolerance ───────────────────────────────────────────────────────

async fn seeded_detector(now: f64) -> StabilityDetector {
    seeded_detector_with(now, REWRITE_TOLERANCE).await
}

async fn seeded_detector_with(now: f64, rewrite_tolerance: f64) -> StabilityDetector {
    let detector = StabilityDetector {
        rewrite_tolerance,
        ..make_detector()
    };
    for i in 0..5 {
        detector.buffer.push(make_candidate(
            FacetClass::Style,
            "verbosity",
            "terse",
            CueFamily::Explicit,
            now - i as f64 * 10.0,
        ));
    }
    detector.rebuild(now).await.unwrap();
    detector
}

async fn verbosity(detector: &StabilityDetector) -> ProfileFacet {
    detector
        .cache
        .list_all()
        .await
        .unwrap()
        .into_iter()
        .find(|f| f.key == "style/verbosity")
        .expect("the facet exists")
}

/// Once a facet has settled, a cycle that brings no evidence moves its scores
/// by a hair, and the row is not written again for it.
///
/// The first cycle after the evidence does change it — that cycle's explicit
/// cues are gone, so its stability and state move — and is written.
#[tokio::test]
async fn a_rebuild_with_nothing_new_leaves_a_settled_row_alone() {
    let now = 1_000_000.0;
    let half_hour = 30.0 * 60.0;
    let detector = seeded_detector(now).await;
    detector.rebuild(now + half_hour).await.unwrap();
    let settled = verbosity(&detector).await;
    assert_eq!(
        settled.last_seen_at,
        now + half_hour,
        "the first cycle writes"
    );

    let outcome = detector.rebuild(now + 2.0 * half_hour).await.unwrap();
    assert_eq!(outcome.kept, 1, "the facet is still kept");
    assert_eq!(
        verbosity(&detector).await,
        settled,
        "nothing moved enough to be written"
    );
}

/// Skipping writes changes nothing a reader acts on. Rebuilding every half
/// hour for a week, a detector that skips rows that moved less than the
/// tolerance keeps, at every step, the lifecycle state of one that writes
/// every row every cycle, with scores within the tolerance — stability
/// included, which counts time since reinforcement rather than decaying from
/// the stored row.
#[tokio::test]
async fn skipped_writes_track_a_detector_that_writes_every_cycle() {
    let start = 1_000_000.0;
    let skipping = seeded_detector_with(start, REWRITE_TOLERANCE).await;
    let writing = seeded_detector_with(start, 0.0).await;
    let mut at = start;
    for _ in 0..(7 * 48) {
        at += 30.0 * 60.0;
        skipping.rebuild(at).await.unwrap();
        writing.rebuild(at).await.unwrap();
        let (skipped, written) = (verbosity(&skipping).await, verbosity(&writing).await);
        let since = at - start;
        assert_eq!(skipped.state, written.state, "state at t+{since}s");
        assert!(
            (skipped.confidence - written.confidence).abs() < REWRITE_TOLERANCE,
            "confidence at t+{since}s: {} vs {}",
            skipped.confidence,
            written.confidence
        );
        assert!(
            (skipped.stability - written.stability).abs() < REWRITE_TOLERANCE,
            "stability at t+{since}s: {} vs {}",
            skipped.stability,
            written.stability
        );
    }
}

#[test]
fn a_row_is_rewritten_for_what_a_reader_would_see() {
    let held = ProfileFacet {
        facet_id: "f".into(),
        facet_type: FacetType::Preference,
        key: "style/verbosity".into(),
        value: "terse".into(),
        confidence: 0.8,
        evidence_count: 5,
        source_segment_ids: None,
        first_seen_at: 1.0,
        last_seen_at: 2.0,
        state: FacetState::Active,
        stability: 1.6,
        user_state: UserState::Auto,
        evidence_refs: Vec::new(),
        class: Some("style".into()),
        cue_families: None,
    };
    let drifted = ProfileFacet {
        confidence: 0.8 - REWRITE_TOLERANCE / 2.0,
        stability: 1.6 - REWRITE_TOLERANCE / 2.0,
        last_seen_at: 99.0,
        ..held.clone()
    };
    assert!(
        !worth_rewriting(&held, &drifted, REWRITE_TOLERANCE),
        "a hair of decay"
    );
    let decayed = ProfileFacet {
        confidence: 0.8 - REWRITE_TOLERANCE,
        ..held.clone()
    };
    assert!(worth_rewriting(&held, &decayed, REWRITE_TOLERANCE));
    let demoted = ProfileFacet {
        state: FacetState::Provisional,
        ..held.clone()
    };
    assert!(worth_rewriting(&held, &demoted, REWRITE_TOLERANCE));
    let reinforced = ProfileFacet {
        evidence_count: 6,
        ..held
    };
    assert!(worth_rewriting(&drifted, &reinforced, REWRITE_TOLERANCE));
}

// ── the rebuild time across restarts ─────────────────────────────────────────

/// A detector over `profile` that keeps its rebuild time in `workspace`.
fn persisted_detector(
    profile: &std::sync::Arc<crate::agent::learning::test_profile::InMemoryProfile>,
    workspace: &std::path::Path,
) -> StabilityDetector {
    StabilityDetector {
        cache: crate::agent::learning::cache::FacetCache::for_tests(profile.clone()),
        ..make_detector()
    }
    .persisted_in(workspace)
}

fn profile_store() -> std::sync::Arc<crate::agent::learning::test_profile::InMemoryProfile> {
    std::sync::Arc::new(crate::agent::learning::test_profile::InMemoryProfile::new())
}

/// Rebuilds `detector` at `start` over explicit evidence for one channel.
async fn seed_channel(detector: &StabilityDetector, start: f64) {
    for i in 0..5 {
        detector.buffer.push(make_candidate(
            FacetClass::Channel,
            "preferred",
            "email",
            CueFamily::Explicit,
            start - i as f64 * 10.0,
        ));
    }
    detector.rebuild(start).await.unwrap();
}

async fn channel_facet(detector: &StabilityDetector) -> Option<ProfileFacet> {
    detector
        .cache
        .list_all()
        .await
        .unwrap()
        .into_iter()
        .find(|f| f.key == "channel/preferred")
}

/// Asserts the channel facet reads alike in both stores on `day`.
async fn assert_same_channel(a: &StabilityDetector, b: &StabilityDetector, day: f64) {
    let (a, b) = (channel_facet(a).await, channel_facet(b).await);
    assert_eq!(
        a.as_ref().map(|f| f.state),
        b.as_ref().map(|f| f.state),
        "state on day {day}"
    );
    if let (Some(a), Some(b)) = (a, b) {
        assert!(
            (a.stability - b.stability).abs() < REWRITE_TOLERANCE,
            "stability on day {day}: {} vs {}",
            a.stability,
            b.stability
        );
    }
}

/// A detector rebuilt over the same store after a restart keeps the
/// lifecycle an uninterrupted one gives. The facet's confidence decays below
/// what the tolerance lets a cycle write, so its row goes weeks without a
/// write; a restarted detector that lost the rebuild time would read those
/// weeks as decay at once and drop the facet.
#[tokio::test]
async fn a_restarted_detector_keeps_the_lifecycle_of_an_uninterrupted_one() {
    let start = 1_000_000.0;
    let step = 6.0 * 3600.0;
    let restarted_dir = tempfile::tempdir().unwrap();
    let uninterrupted_dir = tempfile::tempdir().unwrap();
    let restarted_store = profile_store();
    let uninterrupted_store = profile_store();
    let mut restarted = persisted_detector(&restarted_store, restarted_dir.path());
    let uninterrupted = persisted_detector(&uninterrupted_store, uninterrupted_dir.path());
    seed_channel(&restarted, start).await;
    seed_channel(&uninterrupted, start).await;

    let mut at = start;
    for cycle in 1..=(40 * 4) {
        at += step;
        if cycle == 30 * 4 {
            // The process restarts: a new detector over the same store.
            restarted = persisted_detector(&restarted_store, restarted_dir.path());
        }
        restarted.rebuild(at).await.unwrap();
        uninterrupted.rebuild(at).await.unwrap();
        assert_same_channel(&restarted, &uninterrupted, (at - start) / 86_400.0).await;
    }
    assert!(
        restarted_dir.path().join(REBUILD_STATE_FILE).exists(),
        "the rebuild time is kept in the workspace"
    );
}

/// The scheduled detector lives on beside the one-off detectors the tool and
/// the RPC build over the same workspace, and counts from their rebuilds as
/// from its own. Taking turns with them, it keeps the lifecycle of a detector
/// that ran every rebuild itself; one that read only its own rebuild time
/// would count the days since as decay and drop the facet.
#[tokio::test]
async fn a_long_lived_detector_counts_from_the_one_off_rebuilds_beside_it() {
    let start = 1_000_000.0;
    let step = 6.0 * 3600.0;
    let shared_dir = tempfile::tempdir().unwrap();
    let alone_dir = tempfile::tempdir().unwrap();
    let shared_store = profile_store();
    let alone_store = profile_store();
    let long_lived = persisted_detector(&shared_store, shared_dir.path());
    let alone = persisted_detector(&alone_store, alone_dir.path());
    seed_channel(&long_lived, start).await;
    seed_channel(&alone, start).await;

    let mut at = start;
    for cycle in 1..=(40 * 4) {
        at += step;
        if cycle % 8 == 0 {
            long_lived.rebuild(at).await.unwrap();
        } else {
            let one_off = persisted_detector(&shared_store, shared_dir.path());
            one_off.rebuild(at).await.unwrap();
        }
        alone.rebuild(at).await.unwrap();
        assert_same_channel(&long_lived, &alone, (at - start) / 86_400.0).await;
    }
}

/// Rebuilds over one workspace take turns, whichever detector runs them: a
/// one-off rebuild started while the scheduled one runs waits for it, and
/// then starts from the facets and the time it left.
#[tokio::test]
async fn rebuilds_over_one_workspace_take_turns() {
    let workspace = tempfile::tempdir().unwrap();
    let store = profile_store();
    let scheduled = persisted_detector(&store, workspace.path());
    let one_off = persisted_detector(&store, workspace.path());

    let running = scheduled.rebuild_turn.lock().await;
    let waiting = tokio::spawn(async move { one_off.rebuild(1_000_000.0).await.map(drop) });
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    assert!(
        !waiting.is_finished(),
        "the one-off rebuild ran beside the scheduled one"
    );
    drop(running);
    waiting.await.unwrap().unwrap();
}

/// Rebuilds that finish together leave the latest time stored, whatever
/// order their writes land in, and an earlier rebuild finishing late does
/// not move it back.
#[test]
fn the_stored_rebuild_time_only_moves_forward() {
    let workspace = tempfile::tempdir().unwrap();
    let path = workspace.path().join(REBUILD_STATE_FILE);
    store_rebuild_time(&path, 2_000.0).unwrap();
    store_rebuild_time(&path, 1_000.0).unwrap();
    assert_eq!(read_rebuild_time(&path), Some(2_000.0));

    let writers: Vec<_> = (0..16)
        .map(|i| {
            let path = path.clone();
            std::thread::spawn(move || store_rebuild_time(&path, 3_000.0 + f64::from(i)))
        })
        .collect();
    for writer in writers {
        writer.join().unwrap().unwrap();
    }
    assert_eq!(read_rebuild_time(&path), Some(3_015.0));
}
