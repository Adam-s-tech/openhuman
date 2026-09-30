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
fn a_403_is_a_refused_credential_not_an_expired_session() {
    // An API key without the memory scope is a backend 403. `SESSION_EXPIRED:`
    // would make the app sign the user out; the credential is valid, just not
    // allowed here.
    let refused = "[FORBIDDEN] memory API memory/recall on host (HTTP 403 Forbidden): API key \
                   is missing the required scope: memory — the session expired or the API \
                   key was rejected; re-authenticate";
    let error = anyhow::Error::new(MemoryError::Unauthorized(refused.into()));
    let typed = classify_engine_error(&error);
    assert!(typed.starts_with(MEMORY_FORBIDDEN_PREFIX), "{typed}");
    let text = classify_engine_message(&format!("unauthorized: {refused}"));
    assert!(text.starts_with(MEMORY_FORBIDDEN_PREFIX), "{text}");
    assert!(!text.contains(SESSION_EXPIRED_PREFIX), "{text}");
}

#[test]
fn a_403_counts_only_as_the_unauthorized_errors_own_status() {
    // Each adapter's rendering of a 403: hosted with the backend's code,
    // hosted without one (its default code for a 403 is `UNAUTHORIZED`), and
    // the direct wire. Typed, as text, and behind a caller's context.
    for refused in [
        "[FORBIDDEN] memory API memory/recall on host (HTTP 403 Forbidden): API key is \
         missing the required scope: memory",
        "[UNAUTHORIZED] memory API memory/recall on host (HTTP 403 Forbidden): Forbidden — \
         the session expired or the API key was rejected; re-authenticate",
        "memory API memory/recall on host: the configured credential was rejected (HTTP 403 \
         Forbidden) — check the API key",
    ] {
        let typed = classify_engine_error(&anyhow::Error::new(MemoryError::Unauthorized(
            refused.into(),
        )));
        assert!(typed.starts_with(MEMORY_FORBIDDEN_PREFIX), "{typed}");
        for text in [
            format!("unauthorized: {refused}"),
            format!("memory recall failed: unauthorized: {refused}"),
        ] {
            let classified = classify_engine_message(&text);
            assert!(
                classified.starts_with(MEMORY_FORBIDDEN_PREFIX),
                "{text} -> {classified}"
            );
        }
    }
    // A lapsed session whose body quotes an upstream 403 is still lapsed: the
    // adapter's own status comes first.
    let lapsed = "[UNAUTHORIZED] memory API memory/recall on host (HTTP 401 Unauthorized): \
                  upstream answered (HTTP 403 Forbidden) — the session expired or the API \
                  key was rejected; re-authenticate";
    let typed = classify_engine_error(&anyhow::Error::new(MemoryError::Unauthorized(
        lapsed.into(),
    )));
    assert!(typed.starts_with(SESSION_EXPIRED_PREFIX), "{typed}");
    let text = classify_engine_message(&format!("unauthorized: {lapsed}"));
    assert!(text.starts_with(SESSION_EXPIRED_PREFIX), "{text}");
    // Another error class that quotes a 403 is not a refused credential.
    let other = "invalid input: the upstream answered (HTTP 403 Forbidden)";
    assert_eq!(classify_engine_message(other), other);
}

#[test]
fn an_engine_that_cannot_serve_now_is_unreachable() {
    for error in [
        MemoryError::Unavailable("[RATE_LIMITED] memory API memory/recall (HTTP 429)".into()),
        MemoryError::Unreachable("memory API request to host: could not connect".into()),
        MemoryError::Timeout("memory API request to host: timed out".into()),
    ] {
        let rendered = error.to_string();
        let typed = classify_engine_error(&anyhow::Error::new(error));
        assert!(typed.starts_with(MEMORY_UNREACHABLE_PREFIX), "{typed}");
        let text = classify_engine_message(&rendered);
        assert!(
            text.starts_with(MEMORY_UNREACHABLE_PREFIX),
            "{rendered} -> {text}"
        );
    }
    assert_eq!(
        classify_engine_message("MEMORY_UNREACHABLE: already classified"),
        "MEMORY_UNREACHABLE: already classified"
    );
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

#[test]
fn endpoints_with_userinfo_or_link_local_hosts_are_rejected_without_echoing_them() {
    for bad in [
        "https://user:secret@example.com",
        "https://user@example.com",
        "http://169.254.169.254/latest/meta-data",
        "http://[fe80::1]:8080",
        "http://[::ffff:169.254.169.254]",
        "http://metadata.google.internal",
        "ftp://example.com",
        "not a url",
    ] {
        let err = validate_endpoint(bad).expect_err(bad);
        assert!(
            !err.contains("secret") && !err.contains("user:") && !err.contains("169.254"),
            "the rejection must not echo the endpoint: {err}"
        );
    }
    for good in [
        "https://api.supermemory.ai",
        "http://localhost:3111",
        "http://10.0.0.5:8080",
        "https://[2001:db8::1]/x",
    ] {
        validate_endpoint(good).unwrap_or_else(|e| panic!("{good}: {e}"));
    }
}

#[test]
fn a_rejected_endpoint_never_reaches_prepare_target_output() {
    let mut params = target("supermemory");
    params.endpoint = Some("https://u:p@example.com".into());
    let err = prepare_target(params).unwrap_err();
    assert!(!err.contains("u:p"));
}
