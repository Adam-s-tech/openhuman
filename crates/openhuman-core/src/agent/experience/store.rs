use crate::agent::experience::types::{stable_experience_id, AgentExperience, ExperienceHit};
use crate::memory::safety::sanitize_text;
use crate::memory::{Memory, MemoryCategory};
use base64::Engine as _;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

pub const AGENT_EXPERIENCE_NAMESPACE: &str = "agent_experience";

/// Decode a stored experience payload.
///
/// Records are plain JSON. Rows written before the engine's bare-card gate
/// became corroborated (#6855) were base64(JSON) to dodge a Luhn-valid 13-digit
/// millisecond timestamp being rewritten to `[REDACTED_PII_*]` (#5209); they are
/// still read here. A JSON object starts with `{`, which is not in the base64
/// alphabet, so the base64 decode fails cleanly on plain content and we fall
/// back to parsing it directly: no ambiguity, no migration step.
fn decode_experience_payload(stored: &str) -> Result<AgentExperience, String> {
    if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(stored.trim()) {
        if let Ok(text) = std::str::from_utf8(&bytes) {
            if let Ok(experience) = serde_json::from_str::<AgentExperience>(text) {
                return Ok(experience);
            }
        }
    }
    serde_json::from_str::<AgentExperience>(stored)
        .map_err(|e| format!("parse agent experience: {e}"))
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExperienceQuery {
    pub query: String,
    pub tools: Vec<String>,
    pub tags: Vec<String>,
    pub agent_id: Option<String>,
    pub entrypoint: Option<String>,
    pub max_hits: usize,
}

#[derive(Clone)]
pub struct AgentExperienceStore {
    memory: Arc<dyn Memory>,
}

impl AgentExperienceStore {
    pub fn new(memory: Arc<dyn Memory>) -> Self {
        Self { memory }
    }

    pub async fn put(&self, mut experience: AgentExperience) -> Result<AgentExperience, String> {
        if experience.id.trim().is_empty() {
            experience.id = stable_experience_id(
                &experience.task_summary,
                &experience.tool_sequence,
                experience.outcome,
            );
        }
        if experience.task_summary.trim().is_empty() {
            return Err("task_summary is required".to_string());
        }
        if experience.lesson.trim().is_empty() {
            return Err("lesson is required".to_string());
        }

        let key = storage_key(&experience.id);
        if let Some(existing) = self.fetch(&key).await? {
            experience.created_at_ms = existing.created_at_ms;
        } else if experience.created_at_ms <= 0 {
            experience.created_at_ms = now_ms();
        }
        experience.updated_at_ms = now_ms();
        experience = redact_experience(experience);

        let content = serde_json::to_string(&experience).map_err(|e| e.to_string())?;
        self.memory
            .store(
                AGENT_EXPERIENCE_NAMESPACE,
                &key,
                &content,
                MemoryCategory::Custom(AGENT_EXPERIENCE_NAMESPACE.into()),
                None,
            )
            .await
            .map_err(|e| format!("store agent experience: {e:#}"))?;

        Ok(experience)
    }

    pub async fn list(&self) -> Result<Vec<AgentExperience>, String> {
        let entries = self
            .memory
            .list(Some(AGENT_EXPERIENCE_NAMESPACE), None, None)
            .await
            .map_err(|e| format!("list agent experiences: {e:#}"))?;

        let mut experiences: Vec<AgentExperience> = entries
            .into_iter()
            .filter(|entry| entry.key.starts_with("experience/"))
            .filter_map(|entry| match decode_experience_payload(&entry.content) {
                Ok(experience) => Some(experience),
                Err(err) => {
                    log::warn!(
                        "[agent-experience] skipping malformed entry key={}: {err}",
                        entry.key
                    );
                    None
                }
            })
            .collect();

        experiences.sort_by(|a, b| {
            b.updated_at_ms
                .cmp(&a.updated_at_ms)
                .then_with(|| a.id.cmp(&b.id))
        });
        Ok(experiences)
    }

    pub async fn dismiss(&self, id: &str) -> Result<bool, String> {
        let key = storage_key(id);
        let Some(mut experience) = self.fetch(&key).await? else {
            return Ok(false);
        };
        experience.dismissed = true;
        experience.updated_at_ms = now_ms();
        self.put(experience).await?;
        Ok(true)
    }

    pub async fn retrieve(&self, query: ExperienceQuery) -> Result<Vec<ExperienceHit>, String> {
        if query.max_hits == 0 {
            return Ok(Vec::new());
        }

        let query_terms = terms(&query.query);
        let query_tools = normalized_set(&query.tools);
        let query_tags = normalized_set(&query.tags);

        let mut hits: Vec<ExperienceHit> = self
            .list()
            .await?
            .into_iter()
            .filter(|experience| !experience.dismissed)
            .filter_map(|experience| {
                let (score, match_reasons) = score_experience(
                    &experience,
                    &query_terms,
                    &query_tools,
                    &query_tags,
                    query.agent_id.as_deref(),
                    query.entrypoint.as_deref(),
                );
                (score > 0.0).then_some(ExperienceHit {
                    experience,
                    score,
                    match_reasons,
                })
            })
            .collect();

        hits.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(Ordering::Equal)
                .then_with(|| b.experience.updated_at_ms.cmp(&a.experience.updated_at_ms))
                .then_with(|| a.experience.id.cmp(&b.experience.id))
        });
        hits.truncate(query.max_hits);
        Ok(hits)
    }

    async fn fetch(&self, key: &str) -> Result<Option<AgentExperience>, String> {
        let entry = self
            .memory
            .get(AGENT_EXPERIENCE_NAMESPACE, key)
            .await
            .map_err(|e| format!("get agent experience: {e:#}"))?;
        match entry {
            Some(entry) => decode_experience_payload(&entry.content).map(Some),
            None => Ok(None),
        }
    }
}

/// Retrieve one logical experience pool across multiple physical memory stores.
///
/// Keep the merge, de-duplication, ordering, and final limit in one place so the
/// RPC and live-turn paths cannot drift.
pub async fn retrieve_across_stores(
    stores: &[AgentExperienceStore],
    query: ExperienceQuery,
) -> Result<Vec<ExperienceHit>, String> {
    let max_hits = query.max_hits;
    let mut by_id: BTreeMap<String, ExperienceHit> = BTreeMap::new();
    for store in stores {
        for hit in store.retrieve(query.clone()).await? {
            let id = hit.experience.id.clone();
            match by_id.get(&id) {
                Some(existing) if existing.score >= hit.score => {}
                _ => {
                    by_id.insert(id, hit);
                }
            }
        }
    }
    let mut hits: Vec<_> = by_id.into_values().collect();
    hits.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(Ordering::Equal)
            .then_with(|| b.experience.updated_at_ms.cmp(&a.experience.updated_at_ms))
            .then_with(|| a.experience.id.cmp(&b.experience.id))
    });
    hits.truncate(max_hits);
    Ok(hits)
}

fn storage_key(id: &str) -> String {
    format!("experience/{}", id.trim())
}

/// Redact secrets/PII from every captured free-text field before the record is
/// serialized and stored, so a secret in any of them is gone from both the
/// stored record and what recall returns.
///
/// The full scrubber ([`sanitize_text`]: private-key blocks, Bearer/`sk-`/
/// Stripe/npm/OAuth secrets, national-ID / phone / card PII) runs under the
/// host's corroborated policy, which leaves bare 13-digit epoch-millisecond
/// timestamps alone. Only free-text fields are scrubbed: the numeric
/// timestamp/confidence fields are structural, and `id` is the storage key
/// (scrubbing it would desync key and content), so both stay intact.
fn redact_experience(mut experience: AgentExperience) -> AgentExperience {
    fn scrub(value: &str) -> String {
        sanitize_text(value).value
    }

    experience.task_fingerprint = scrub(&experience.task_fingerprint);
    experience.task_summary = scrub(&experience.task_summary);
    experience.lesson = scrub(&experience.lesson);
    experience.reuse_hint = scrub(&experience.reuse_hint);
    experience.avoid_hint = experience.avoid_hint.as_deref().map(scrub);
    experience.error_class = experience.error_class.as_deref().map(scrub);
    experience.agent_id = experience.agent_id.as_deref().map(scrub);
    experience.entrypoint = experience.entrypoint.as_deref().map(scrub);
    experience.tools_used = experience.tools_used.iter().map(|t| scrub(t)).collect();
    experience.tool_sequence = experience.tool_sequence.iter().map(|t| scrub(t)).collect();
    experience.tags = experience.tags.iter().map(|t| scrub(t)).collect();
    experience
}

fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

fn score_experience(
    experience: &AgentExperience,
    query_terms: &BTreeSet<String>,
    query_tools: &BTreeSet<String>,
    query_tags: &BTreeSet<String>,
    agent_id: Option<&str>,
    entrypoint: Option<&str>,
) -> (f32, Vec<String>) {
    let mut score = experience.confidence.clamp(0.0, 1.0) * 0.2;
    let mut reasons = Vec::new();

    let experience_tools = normalized_set(&experience.tools_used);
    let tool_overlap = overlap_count(query_tools, &experience_tools);
    if tool_overlap > 0 {
        score += 3.0 + tool_overlap as f32 * 0.5;
        reasons.push("tool_overlap".to_string());
    }

    let experience_tags = normalized_set(&experience.tags);
    let tag_overlap = overlap_count(query_tags, &experience_tags);
    if tag_overlap > 0 {
        score += 2.0 + tag_overlap as f32 * 0.25;
        reasons.push("tag_overlap".to_string());
    }

    let haystack = terms(&format!(
        "{} {} {} {}",
        experience.task_summary,
        experience.lesson,
        experience.reuse_hint,
        experience.avoid_hint.as_deref().unwrap_or_default()
    ));
    let query_overlap = overlap_count(query_terms, &haystack);
    if query_overlap > 0 {
        score += 1.0 + query_overlap as f32 * 0.2;
        reasons.push("query_overlap".to_string());
    }

    if let (Some(query_agent), Some(exp_agent)) = (agent_id, experience.agent_id.as_deref()) {
        if normalize(query_agent) == normalize(exp_agent) {
            score += 1.0;
            reasons.push("agent_match".to_string());
        }
    }

    if let (Some(query_entrypoint), Some(exp_entrypoint)) =
        (entrypoint, experience.entrypoint.as_deref())
    {
        if normalize(query_entrypoint) == normalize(exp_entrypoint) {
            score += 0.5;
            reasons.push("entrypoint_match".to_string());
        }
    }

    (score, reasons)
}

fn normalized_set(values: &[String]) -> BTreeSet<String> {
    values
        .iter()
        .map(|value| normalize(value))
        .filter(|value| !value.is_empty())
        .collect()
}

fn terms(input: &str) -> BTreeSet<String> {
    input
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .map(normalize)
        .filter(|term| term.len() > 2)
        .collect()
}

fn overlap_count(a: &BTreeSet<String>, b: &BTreeSet<String>) -> usize {
    a.intersection(b).count()
}

fn normalize(input: &str) -> String {
    input.trim().to_ascii_lowercase()
}

#[cfg(test)]
#[path = "store_tests.rs"]
mod tests;
