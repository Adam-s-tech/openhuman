use super::Pending;
use crate::security::approval::{ApprovalGate, GateOutcome};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub(super) fn task_inputs(args: &Value) -> anyhow::Result<BTreeMap<String, String>> {
    args["inputs"].as_object().map_or_else(
        || Ok(BTreeMap::new()),
        |inputs| {
            inputs
                .iter()
                .map(|(k, v)| {
                    Ok((
                        k.clone(),
                        v.as_str()
                            .ok_or_else(|| anyhow::anyhow!("Task input '{k}' must be text"))?
                            .to_owned(),
                    ))
                })
                .collect()
        },
    )
}

/// Ask the host approval gate about a paused task's exact action. A missing
/// gate denies: a task never takes an irreversible step unapproved.
pub(super) async fn approve_task_action(pending: &Pending) -> anyhow::Result<bool> {
    let gate = ApprovalGate::try_global().ok_or_else(|| {
        anyhow::anyhow!("[policy-denied] Browser action needs an interactive host approval gate")
    })?;
    let clean = |raw: &str| {
        let cleaned = raw.chars().filter(|c| !c.is_control()).collect::<String>();
        let mut short = cleaned.chars().take(160).collect::<String>();
        if cleaned.chars().count() > 160 {
            short.push('…');
        }
        short
    };
    let digest = Sha256::digest(serde_json::to_vec(&json!({
        "task": pending.task, "action": pending.action, "target": pending.target
    }))?);
    let digest_hex = format!("{digest:x}");
    let summary = format!(
        "Browser task: {} — {} [action {}]",
        clean(&pending.action),
        clean(&pending.target),
        &digest_hex[..12]
    );
    let args = json!({"action": "task_step", "target": summary, "exact_action_sha256": digest_hex});
    Ok(
        match gate.intercept_forced("browser", &summary, args).await {
            GateOutcome::Allow => true,
            GateOutcome::Deny { reason } => {
                tracing::debug!(%reason, "[browser] task action denied by host");
                false
            }
        },
    )
}
