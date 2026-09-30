//! Install-time location of a skills.sh entry's `SKILL.md`.
//!
//! The per-source URL knowledge (Hermes `docsPath`, GitHub blob/tree, ClawHub,
//! skills.sh candidate paths, tree-listing lookup) lives in
//! [`tinyskills`]'s catalog module. This host module keeps only what needs the
//! network: probing the conventional locations and, failing that, listing the
//! repo tree once.

use std::time::Duration;

use serde_json::Value;
use tinyskills::SkillsShRef;

/// Test override for every catalog entry's download base.
pub(super) const DOWNLOAD_BASE_URL_ENV: &str = "OPENHUMAN_SKILL_REGISTRY_DOWNLOAD_BASE_URL";
const PROBE_TIMEOUT_SECS: u64 = 15;

/// The download-base override from the environment, if set and non-empty.
pub(super) fn download_base_override() -> Option<String> {
    std::env::var(DOWNLOAD_BASE_URL_ENV)
        .ok()
        .filter(|base| !base.trim().is_empty())
}

/// Locate a skills.sh skill's `SKILL.md` in its GitHub repo.
pub(super) async fn resolve_skills_sh(skill: &SkillsShRef<'_>) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(PROBE_TIMEOUT_SECS))
        .user_agent("openhuman-core")
        .build()
        .map_err(|e| format!("failed to build http client: {e}"))?;

    // Probe every conventional location at once: probed one after another,
    // each slow miss could spend the whole timeout before the next starts.
    let candidates = skill.candidate_urls();
    let responses =
        futures::future::join_all(candidates.iter().map(|url| client.head(url).send())).await;
    for (url, response) in candidates.into_iter().zip(responses) {
        match response {
            Ok(resp) if resp.status().is_success() => {
                tracing::info!(url = %url, "[skill_registry] skills.sh SKILL.md found");
                return Ok(url);
            }
            Ok(resp) => tracing::debug!(
                url = %url,
                status = resp.status().as_u16(),
                "[skill_registry] skills.sh candidate missing"
            ),
            Err(error) => tracing::debug!(
                url = %url,
                error = %error,
                "[skill_registry] skills.sh candidate probe failed"
            ),
        }
    }

    // Not in a conventional directory: one recursive listing finds it anywhere.
    let repo = skill.repo_label();
    let name = skill.skill;
    tracing::info!(repo = %repo, skill = %name, "[skill_registry] listing repo tree for skills.sh skill");
    let resp = client
        .get(skill.tree_api_url())
        .send()
        .await
        .map_err(|e| format!("could not list {repo} to locate '{name}': {e}"))?;
    if !resp.status().is_success() {
        return Err(format!(
            "could not list {repo} to locate '{name}' (GitHub returned {})",
            resp.status().as_u16()
        ));
    }
    let tree: Value = resp
        .json()
        .await
        .map_err(|e| format!("could not read the {repo} file listing: {e}"))?;
    skill
        .locate_in_tree(&tree)
        .map_err(|miss| skill.miss_message(&miss))
}
