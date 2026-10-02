use super::*;

#[test]
fn env_overlay_auto_update_interval_parses_u32() {
    let mut cfg = Config::default();
    cfg.apply_env_overlay_with(
        &HashMapEnv::new()
            .with("OPENHUMAN_AUTO_UPDATE_ENABLED", "true")
            .with("OPENHUMAN_AUTO_UPDATE_INTERVAL_MINUTES", "60"),
    );
    assert!(cfg.update.enabled);
    assert_eq!(cfg.update.interval_minutes, 60);

    // Garbage numeric — ignored, previous value retained.
    cfg.apply_env_overlay_with(
        &HashMapEnv::new().with("OPENHUMAN_AUTO_UPDATE_INTERVAL_MINUTES", "hello"),
    );
    assert_eq!(cfg.update.interval_minutes, 60);
}

#[test]
fn env_overlay_auto_update_restart_strategy_accepts_supported_values() {
    let mut cfg = Config::default();
    cfg.apply_env_overlay_with(
        &HashMapEnv::new().with("OPENHUMAN_AUTO_UPDATE_RESTART_STRATEGY", "supervisor"),
    );
    assert_eq!(
        cfg.update.restart_strategy,
        crate::config::UpdateRestartStrategy::Supervisor
    );

    cfg.apply_env_overlay_with(
        &HashMapEnv::new().with("OPENHUMAN_AUTO_UPDATE_RESTART_STRATEGY", "self_replace"),
    );
    assert_eq!(
        cfg.update.restart_strategy,
        crate::config::UpdateRestartStrategy::SelfReplace
    );
}

#[test]
fn env_overlay_tool_dispatcher_overrides_the_agent_field_when_non_blank() {
    let mut cfg = Config::default();
    assert_eq!(cfg.agent.tool_dispatcher, "python");

    cfg.apply_env_overlay_with(&HashMapEnv::new().with("OPENHUMAN_TOOL_DISPATCHER", " native "));
    assert_eq!(cfg.agent.tool_dispatcher, "native");

    // Blank values leave the persisted choice alone.
    cfg.apply_env_overlay_with(&HashMapEnv::new().with("OPENHUMAN_TOOL_DISPATCHER", "   "));
    assert_eq!(cfg.agent.tool_dispatcher, "native");
    cfg.apply_env_overlay_with(&HashMapEnv::new().with("OPENHUMAN_TOOL_DISPATCHER", ""));
    assert_eq!(cfg.agent.tool_dispatcher, "native");
}

#[test]
fn env_overlay_jev_route_and_base_url_override_tool_search_when_non_blank() {
    let mut cfg = Config::default();
    assert_eq!(cfg.agent.tool_search.jev_route, "auto");
    assert_eq!(cfg.agent.tool_search.jev_base_url, None);

    cfg.apply_env_overlay_with(
        &HashMapEnv::new()
            .with("OPENHUMAN_JEV_ROUTE", " OpenRouter ")
            .with("OPENHUMAN_JEV_BASE_URL", " http://127.0.0.1:18080 "),
    );
    assert_eq!(cfg.agent.tool_search.jev_route, "openrouter");
    assert_eq!(
        cfg.agent.tool_search.jev_base_url.as_deref(),
        Some("http://127.0.0.1:18080")
    );

    // Blank values leave the persisted choice alone.
    cfg.apply_env_overlay_with(
        &HashMapEnv::new()
            .with("OPENHUMAN_JEV_ROUTE", "  ")
            .with("OPENHUMAN_JEV_BASE_URL", ""),
    );
    assert_eq!(cfg.agent.tool_search.jev_route, "openrouter");
    assert_eq!(
        cfg.agent.tool_search.jev_base_url.as_deref(),
        Some("http://127.0.0.1:18080")
    );
}

/// Local model tier presets were removed: OpenHuman no longer picks models by
/// RAM tier. A stale `OPENHUMAN_LOCAL_AI_TIER` in the environment must be
/// ignored rather than rewriting the user's configured local models.
#[test]
fn env_overlay_ignores_removed_local_ai_tier_var() {
    let env = HashMapEnv::new().with("OPENHUMAN_LOCAL_AI_TIER", "ram_2_4gb");
    let mut cfg = Config::default();
    cfg.local_ai.chat_model_id = "llama3.1:8b".to_string();
    cfg.local_ai.embedding_model_id = "nomic-embed-text:latest".to_string();
    cfg.apply_env_overlay_with(&env);
    assert_eq!(cfg.local_ai.chat_model_id, "llama3.1:8b");
    assert_eq!(cfg.local_ai.embedding_model_id, "nomic-embed-text:latest");
}

/// A config.toml written while OpenHuman still downloaded local models carries
/// tier, quantization, preload, binary-path and download-URL keys under
/// `[local_ai]`. Those keys are no longer read, but such a file must still
/// load with the user's endpoint and model choices intact.
#[test]
fn legacy_local_ai_download_keys_still_load() {
    let legacy = r#"
api_url = "http://127.0.0.1:9"

[local_ai]
runtime_enabled = true
opt_in_confirmed = true
provider = "ollama"
base_url = "http://127.0.0.1:11434"
chat_model_id = "llama3.1:8b"
embedding_model_id = "bge-m3"
selected_tier = "ram_2_4gb"
quantization = "q4_k_m"
preload_vision_model = true
preload_embedding_model = true
preload_stt_model = false
preload_tts_voice = false
ollama_binary_path = "/opt/openhuman/bin/ollama"
download_url = "https://example.invalid/model.gguf"
stt_download_url = "https://example.invalid/stt.bin"
tts_download_url = "https://example.invalid/voice.onnx"
tts_config_download_url = "https://example.invalid/voice.onnx.json"
"#;
    let cfg: Config = toml::from_str(legacy).expect("legacy local_ai keys must still parse");
    assert!(cfg.local_ai.runtime_enabled);
    assert!(cfg.local_ai.opt_in_confirmed);
    assert_eq!(cfg.local_ai.provider, "ollama");
    assert_eq!(
        cfg.local_ai.base_url.as_deref(),
        Some("http://127.0.0.1:11434")
    );
    assert_eq!(cfg.local_ai.chat_model_id, "llama3.1:8b");
    assert_eq!(cfg.local_ai.embedding_model_id, "bge-m3");

    // The runtime projection carries only endpoint and model settings.
    let runtime = crate::inference::local_runtime_config(&cfg);
    assert_eq!(runtime.local_ai.chat_model_id, "llama3.1:8b");
    assert_eq!(
        runtime.local_ai.base_url.as_deref(),
        Some("http://127.0.0.1:11434")
    );
}
