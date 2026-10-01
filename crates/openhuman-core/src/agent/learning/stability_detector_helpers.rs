//! Private aggregation helpers for the stability detector.

use std::collections::{HashMap, HashSet};

use super::{half_life, TAU_EVICT, TAU_PROMOTE, TAU_PROVISIONAL};
use crate::agent::learning::candidate::{self, CueFamily, FacetClass, LearningCandidate};
use tinymemory_api::provider::{FacetState, ProfileFacet, UserState};

pub(super) fn later(a: Option<f64>, b: Option<f64>) -> Option<f64> {
    match (a, b) {
        (Some(a), Some(b)) => Some(a.max(b)),
        (a, b) => a.or(b),
    }
}

// ── Rebuild internals ─────────────────────────────────────────────────────────

pub(super) struct ComputedFacet {
    pub(super) is_new: bool,
    pub(super) facet: ProfileFacet,
}

/// Choose the winning value for a `(class, key)` group via `argmax(stability)`.
///
/// Returns the value with the highest combined evidence weight. Falls back to the
/// existing row's value if no candidates are present.
pub(super) fn select_winning_value(
    cands: &[LearningCandidate],
    existing: Option<&ProfileFacet>,
    now: f64,
    class: FacetClass,
) -> Option<String> {
    if cands.is_empty() {
        return existing.map(|f| f.value.clone());
    }

    // Score each distinct value.
    let mut value_scores: HashMap<&str, f64> = HashMap::new();
    for c in cands {
        let dt = (now - c.observed_at).max(0.0);
        let recency = (-dt / half_life(class)).exp();
        let score = c.cue_family.weight() * recency * c.initial_confidence;
        *value_scores.entry(c.value.as_str()).or_default() += score;
    }

    // If existing row matches a candidate value, add its weight too.
    if let Some(existing) = existing {
        let dt = (now - existing.last_seen_at).max(0.0);
        let recency = (-dt / half_life(class)).exp();
        let existing_score = recency * existing.confidence;
        *value_scores.entry(existing.value.as_str()).or_default() += existing_score;
    }

    value_scores
        .into_iter()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(v, _)| v.to_string())
}

/// Whether a recomputed facet says something its stored row does not: any
/// field other than the two scores, or a score that moved by at least
/// `tolerance` ([`REWRITE_TOLERANCE`] in production). `last_seen_at` is left
/// out on purpose — see the constant's docs.
pub(crate) fn worth_rewriting(
    held: &ProfileFacet,
    computed: &ProfileFacet,
    tolerance: f64,
) -> bool {
    held.value != computed.value
        || held.state != computed.state
        || held.user_state != computed.user_state
        || held.facet_type != computed.facet_type
        || held.evidence_count != computed.evidence_count
        || held.evidence_refs != computed.evidence_refs
        || held.source_segment_ids != computed.source_segment_ids
        || held.class != computed.class
        || held.cue_families != computed.cue_families
        || held.first_seen_at != computed.first_seen_at
        || (held.confidence - computed.confidence).abs() >= tolerance
        || (held.stability - computed.stability).abs() >= tolerance
}

/// Aggregate stability contribution from all candidates (not per-value).
/// Returns (aggregate_score, has_explicit_evidence).
pub(super) fn aggregate_stability(
    cands: &[LearningCandidate],
    existing: Option<&ProfileFacet>,
    now: f64,
    class: FacetClass,
) -> (f64, bool) {
    let mut score = 0.0f64;
    let mut has_explicit = false;

    for c in cands {
        let dt = (now - c.observed_at).max(0.0);
        let recency = (-dt / half_life(class)).exp();
        score += c.cue_family.weight() * recency;
        if matches!(c.cue_family, CueFamily::Explicit) {
            has_explicit = true;
        }
    }

    if let Some(existing) = existing {
        let dt = (now - existing.last_seen_at).max(0.0);
        let recency = (-dt / half_life(class)).exp();
        score += existing.confidence * recency;
    }

    (score, has_explicit)
}

/// Determine the dominant cue family (highest weight).
pub(super) fn dominant_cue(
    cands: &[LearningCandidate],
    _existing: Option<&ProfileFacet>,
) -> CueFamily {
    cands
        .iter()
        .max_by(|a, b| {
            a.cue_family
                .weight()
                .partial_cmp(&b.cue_family.weight())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|c| c.cue_family)
        .unwrap_or(CueFamily::Behavioral)
}

/// Convert the learning domain's `EvidenceRef` to the memory contract's.
///
/// # Why a conversion and not one type
///
/// They are the *same shape* — `memory/api/host/evidence.rs` and
/// `tinymemory-api`'s copy are byte-identical, and this round-trips through
/// serde precisely because of that. They are nominally distinct only because
/// the learning candidate types still live in `tinymemory_core`, so
/// `candidate::EvidenceRef` resolves to the crate's copy while
/// `ProfileFacet::evidence_refs` uses the host's.
///
/// This bridge disappears when `learning_candidate` comes home — it is agent
/// domain knowledge, not engine storage, and belongs host-side with the rest of
/// the learning subsystem. Tracked as stage 4 in
/// `docs/specs/2026-08-13-memory-module-port.md`.
pub(super) fn evidence_to_contract(
    refs: &[candidate::EvidenceRef],
) -> Vec<tinymemory_api::host::EvidenceRef> {
    refs.iter()
        .filter_map(|r| {
            serde_json::to_value(r)
                .ok()
                .and_then(|v| serde_json::from_value(v).ok())
        })
        .collect()
}

/// The inverse of [`evidence_to_contract`].
pub(super) fn evidence_from_contract(
    refs: &[tinymemory_api::host::EvidenceRef],
) -> Vec<candidate::EvidenceRef> {
    refs.iter()
        .filter_map(|r| {
            serde_json::to_value(r)
                .ok()
                .and_then(|v| serde_json::from_value(v).ok())
        })
        .collect()
}

/// Merge the existing row's evidence refs with this cycle's new refs,
/// deduplicating while preserving first-seen order.
///
/// `Vec::dedup_by` only collapses *consecutive* equal elements, so a ref that
/// recurs non-adjacently — present in the existing row and re-emitted by a new
/// candidate, or repeated within one cycle — would slip through and accumulate
/// without bound across rebuilds. `EvidenceRef: Eq + Hash`, so tracking seen
/// refs in a set removes every duplicate exactly and cheaply.
pub(super) fn merge_evidence_refs(
    existing_refs: &[candidate::EvidenceRef],
    new_refs: Vec<candidate::EvidenceRef>,
) -> Vec<candidate::EvidenceRef> {
    let mut seen: HashSet<candidate::EvidenceRef> = HashSet::new();
    existing_refs
        .iter()
        .cloned()
        .chain(new_refs)
        .filter(|r| seen.insert(r.clone()))
        .collect()
}

/// Total evidence count from candidates + existing row.
pub(super) fn total_evidence_count(
    cands: &[LearningCandidate],
    existing: Option<&ProfileFacet>,
) -> u32 {
    let from_existing = existing.map(|f| f.evidence_count as u32).unwrap_or(0);
    from_existing + cands.len() as u32
}

/// The most recent observation timestamp across candidates and the existing row.
///
/// The result is floored at `now - half_life(class)` so a facet's recency decay
/// in [`stability`] bottoms out at one (class-specific) half-life. The floor
/// must use the facet's own `class`: every other per-group computation in
/// `rebuild` is class-scoped, and the half-lives span 7d (Channel) to 90d
/// (Identity), so a hardcoded class would over-retain longer-lived facets and
/// evict shorter-lived ones too early.
/// An existing row counts as reinforced when it was last written, or at the
/// previous rebuild (`refreshed_at`), whichever is later: that rebuild would
/// have rewritten the row before rows that barely moved were skipped.
pub(super) fn most_recent_reinforcement(
    cands: &[LearningCandidate],
    existing: Option<&ProfileFacet>,
    refreshed_at: Option<f64>,
    now: f64,
    class: FacetClass,
) -> f64 {
    let newest_cand = cands
        .iter()
        .map(|c| c.observed_at)
        .fold(f64::NEG_INFINITY, f64::max);
    let existing_ts = existing
        .map(|f| {
            f.last_seen_at
                .max(refreshed_at.unwrap_or(f64::NEG_INFINITY))
        })
        .unwrap_or(f64::NEG_INFINITY);
    newest_cand.max(existing_ts).max(now - half_life(class))
}

/// Map a stability score + user_state to a lifecycle state.
pub(super) fn state_from_stability(score: f64, user_state: UserState) -> FacetState {
    // Pinned → always Active; Forgotten → always Dropped.
    if matches!(user_state, UserState::Pinned) {
        return FacetState::Active;
    }
    if matches!(user_state, UserState::Forgotten) {
        return FacetState::Dropped;
    }

    if score.is_infinite() || score >= TAU_PROMOTE {
        FacetState::Active
    } else if score >= TAU_PROVISIONAL {
        FacetState::Provisional
    } else if score >= TAU_EVICT {
        FacetState::Candidate
    } else {
        FacetState::Dropped
    }
}

/// Canonical key prefix string for a class (used when grouping candidates).
pub(super) fn class_prefix(class: FacetClass) -> &'static str {
    crate::agent::learning::cache::class_prefix(class)
}
