use super::tests::{make_candidate, make_detector};
use super::*;
use crate::agent::learning::candidate::FacetClass;

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
