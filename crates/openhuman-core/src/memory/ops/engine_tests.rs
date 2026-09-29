use super::*;

use crate::memory::api::error::MemoryError;

fn target(driver: &str) -> EngineTargetParams {
    EngineTargetParams {
        driver: driver.to_string(),
        endpoint: None,
        deployment: None,
        api_key: None,
    }
}

#[test]
fn the_module_is_listed_first_and_null_is_never_offered() {
    let engines = available_engines();
    assert_eq!(engines[0].id, MODULE_ID);
    assert_eq!(engines[0].label, "TinyCortex (local)");
    assert!(engines.iter().all(|e| e.id != "null"));
    assert!(
        engines.iter().all(|e| e.id != "tinycortex"),
        "the factory's in-memory tinycortex is not persistent and must not be offered"
    );
}

#[test]
fn the_module_advertises_every_capability_family() {
    let module = &available_engines()[0];
    let all: Vec<String> = crate::memory::api::capabilities::Capabilities::all()
        .iter()
        .map(|c| c.as_str().to_string())
        .collect();
    assert_eq!(module.capabilities, all);
}

#[cfg(feature = "memory-remote")]
#[test]
fn every_remote_engine_is_listed_with_at_least_the_mandatory_families() {
    let engines = available_engines();
    for id in [
        "tinyhumans",
        "supermemory",
        "mem0",
        "cognee",
        "cortex",
        "agentmemory",
    ] {
        let engine = engines
            .iter()
            .find(|e| e.id == id)
            .unwrap_or_else(|| panic!("{id} missing"));
        for family in ["core", "recall", "portability"] {
            assert!(
                engine.capabilities.iter().any(|c| c == family),
                "{id} must advertise {family}: {:?}",
                engine.capabilities
            );
        }
    }
    let hosted = engines.iter().find(|e| e.id == "tinyhumans").unwrap();
    assert!(hosted.hosted);
    assert!(!hosted.needs_key);
    assert!(hosted.capabilities.iter().any(|c| c == "answer"));
}

#[test]
fn prepare_rejects_empty_null_and_unknown_engines() {
    for bad in ["", "   ", "null", "not-an-engine"] {
        assert!(
            prepare_target(target(bad)).is_err(),
            "{bad:?} must be rejected"
        );
    }
}

#[test]
fn prepare_folds_the_legacy_alias_and_ignores_module_options() {
    let mut params = target("tinycortex");
    params.endpoint = Some("https://ignored.example".into());
    params.api_key = Some("ignored".into());
    let prepared = prepare_target(params).expect("the alias names the module");
    assert_eq!(prepared.id, MODULE_ID);
    assert!(prepared.endpoint.is_none() && prepared.api_key.is_none());
    assert!(!prepared.takes_key);
}

#[cfg(feature = "memory-remote")]
#[test]
fn prepare_validates_endpoint_and_deployment() {
    let mut bad_endpoint = target("supermemory");
    bad_endpoint.endpoint = Some("ftp://nope".into());
    assert!(prepare_target(bad_endpoint)
        .unwrap_err()
        .contains("http(s)"));

    let mut bad_deployment = target("mem0");
    bad_deployment.deployment = Some("moon".into());
    assert!(prepare_target(bad_deployment)
        .unwrap_err()
        .contains("deployment"));

    let mut good = target("mem0");
    good.endpoint = Some(" https://api.mem0.ai ".into());
    good.deployment = Some("cloud".into());
    good.api_key = Some("  m0-key  ".into());
    let prepared = prepare_target(good).unwrap();
    assert_eq!(prepared.endpoint.as_deref(), Some("https://api.mem0.ai"));
    assert_eq!(prepared.deployment.as_deref(), Some("cloud"));
    assert_eq!(prepared.api_key.as_deref(), Some("m0-key"));
    assert!(prepared.takes_key);
}

#[cfg(feature = "memory-remote")]
#[test]
fn the_hosted_engine_drops_a_caller_supplied_endpoint_and_key() {
    let mut params = target("tinyhumans");
    params.endpoint = Some("https://attacker.example".into());
    params.api_key = Some("tiny_live_pasted".into());
    params.deployment = Some("cloud".into());
    let prepared = prepare_target(params).unwrap();
    assert!(
        prepared.endpoint.is_none(),
        "an endpoint could redirect the session bearer"
    );
    assert!(prepared.api_key.is_none());
    assert!(prepared.deployment.is_none());
    assert!(
        !prepared.takes_key,
        "the hosted entry carries no credential_ref"
    );
}

#[test]
fn a_402_from_the_hosted_backend_becomes_insufficient_credits() {
    let error = anyhow::Error::new(MemoryError::BudgetExceeded(
        "[USER_INSUFFICIENT_CREDITS] memory API memory/experience on host (HTTP 402): x".into(),
    ));
    assert!(classify_engine_error(&error).starts_with(INSUFFICIENT_CREDITS_PREFIX));
}

#[test]
fn a_budget_error_without_the_credits_code_is_left_alone() {
    let error = anyhow::Error::new(MemoryError::BudgetExceeded("token budget".into()));
    let message = classify_engine_error(&error);
    assert!(
        !message.starts_with(INSUFFICIENT_CREDITS_PREFIX),
        "{message}"
    );
}

#[test]
fn unauthorized_becomes_session_expired() {
    let error = anyhow::Error::new(MemoryError::Unauthorized("rejected".into()));
    assert!(classify_engine_error(&error).starts_with(SESSION_EXPIRED_PREFIX));
}

#[test]
fn backend_unavailable_and_prefixed_messages_pass_through() {
    let unavailable = format!(
        "{} no backend transport installed",
        crate::core::observability::BACKEND_UNAVAILABLE_PREFIX
    );
    assert_eq!(classify_engine_message(&unavailable), unavailable);
    assert_eq!(
        classify_engine_message("SESSION_EXPIRED: no TinyHumans session"),
        "SESSION_EXPIRED: no TinyHumans session"
    );
    assert_eq!(classify_engine_message("plain"), "plain");
}

#[test]
fn a_bearer_source_failure_surfaces_as_session_expired() {
    // The hosted adapter wraps a failing `BearerSource` in `Unauthorized`; the
    // message the bearer reports keeps the prefix so string paths agree too.
    assert!(classify_engine_message(
        "SESSION_EXPIRED: no TinyHumans session (credential lookup failed)"
    )
    .starts_with(SESSION_EXPIRED_PREFIX));
    assert!(
        classify_engine_message("unauthorized: SESSION_EXPIRED: no TinyHumans session")
            .starts_with(SESSION_EXPIRED_PREFIX)
    );
}

#[test]
fn state_and_list_serialise_to_the_bare_wire_shapes_the_ui_reads() {
    let state = serde_json::to_value(EngineState {
        driver: "tinymemory".into(),
        endpoint: None,
        deployment: None,
        has_credential: false,
        class: "module".into(),
        fell_back_from: None,
        last_error: None,
    })
    .unwrap();
    let mut keys: Vec<&str> = state
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort_unstable();
    assert_eq!(
        keys,
        [
            "class",
            "deployment",
            "driver",
            "endpoint",
            "fell_back_from",
            "has_credential",
            "last_error"
        ]
    );
    let list = serde_json::to_value(EnginesList {
        engines: available_engines(),
        active: "tinymemory".into(),
    })
    .unwrap();
    let engine = &list["engines"][0];
    for key in [
        "id",
        "label",
        "description",
        "needs_endpoint",
        "needs_key",
        "key_optional",
        "deployments",
        "default_endpoint",
        "hosted",
        "capabilities",
    ] {
        assert!(
            engine.get(key).is_some(),
            "engine descriptor lacks {key}: {engine}"
        );
    }
}
