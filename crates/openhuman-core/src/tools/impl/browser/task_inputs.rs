use std::collections::BTreeMap;

use serde_json::Value;

pub(super) fn task_inputs(args: &Value) -> anyhow::Result<BTreeMap<String, String>> {
    args["inputs"].as_object().map_or_else(
        || Ok(BTreeMap::new()),
        |inputs| {
            inputs
                .iter()
                .map(|(key, value)| {
                    Ok((
                        key.clone(),
                        value
                            .as_str()
                            .ok_or_else(|| anyhow::anyhow!("Task input '{key}' must be text"))?
                            .to_owned(),
                    ))
                })
                .collect()
        },
    )
}
