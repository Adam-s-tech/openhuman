use super::*;

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

fn required<'a>(args: &'a Value, key: &str) -> anyhow::Result<&'a str> {
    args.get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| anyhow::anyhow!("Missing '{key}' parameter"))
}

pub(super) fn parse_action(args: &Value) -> anyhow::Result<Action> {
    let target = || required(args, "selector").map(Target::parse);
    Ok(match required(args, "action")? {
        "click" => Action::Click {
            target: target()?,
            new_tab: false,
        },
        "fill" => Action::Fill {
            target: target()?,
            value: required(args, "value")?.into(),
        },
        "type" => Action::Type {
            target: args["selector"].as_str().map(Target::parse),
            text: required(args, "text")?.into(),
            delay_ms: None,
        },
        "get_text" => Action::GetText { target: target()? },
        "is_visible" => Action::IsVisible { target: target()? },
        "hover" => Action::Hover { target: target()? },
        "press" => Action::Press {
            key: required(args, "key")?.into(),
        },
        "scroll" => Action::Scroll {
            direction: match required(args, "direction")? {
                "up" => ScrollDirection::Up,
                "down" => ScrollDirection::Down,
                "left" => ScrollDirection::Left,
                "right" => ScrollDirection::Right,
                x => anyhow::bail!("Invalid direction: {x}"),
            },
            pixels: args["pixels"].as_u64().and_then(|v| u32::try_from(v).ok()),
            target: None,
        },
        "wait" => Action::WaitFor {
            target: args["selector"].as_str().map(Target::parse),
            text: args["text"].as_str().map(str::to_owned),
            state: WaitState::Visible,
            ms: args["ms"].as_u64(),
            timeout_ms: args["timeout_ms"].as_u64(),
        },
        "find" => {
            let by = match required(args, "by")? {
                "role" => LocateBy::Role,
                "text" => LocateBy::Text,
                "label" => LocateBy::Label,
                "placeholder" => LocateBy::Placeholder,
                "testid" => LocateBy::TestId,
                x => anyhow::bail!("Invalid locator: {x}"),
            };
            let target = Target::locator(Locator::new(by, required(args, "value")?));
            match required(args, "find_action")? {
                "click" => Action::Click {
                    target,
                    new_tab: false,
                },
                "fill" => Action::Fill {
                    target,
                    value: required(args, "fill_value")?.into(),
                },
                "text" => Action::GetText { target },
                "hover" => Action::Hover { target },
                x => anyhow::bail!("Invalid find action: {x}"),
            }
        }
        x => anyhow::bail!("Unsupported browser action: {x}"),
    })
}

