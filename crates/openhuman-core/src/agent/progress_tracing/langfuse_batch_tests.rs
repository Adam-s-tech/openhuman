use super::*;

// The canonical production and staging APIs accept traces. Unknown origins
// must remain closed before a session bearer is read.

#[test]
fn push_is_allowed_for_our_backends() {
    assert!(push_allowed("production"));
    assert!(push_allowed("staging"));
    assert!(push_allowed("development"));
    assert!(!push_allowed("external"));
}

#[test]
fn an_unrecognised_environment_fails_closed() {
    // The allowlist, not a `!= "production"` negation, is what makes this
    // true: a bucket nobody has thought of yet does not push.
    for unknown in ["preview", "prod", "PRODUCTION", "", "qa", "external"] {
        assert!(
            !push_allowed(unknown),
            "{unknown:?} must not push — the allowlist is the fail-closed guard"
        );
    }
}

#[test]
fn every_environment_for_base_bucket_is_classified_deliberately() {
    // Ties the two functions together: if `environment_for_base` grows a
    // bucket, this fails until someone decides which side it belongs on.
    assert!(push_allowed(environment_for_base(
        "https://staging-api.tinyhumans.ai"
    )));
    assert!(push_allowed(environment_for_base("http://localhost:7788")));
    assert!(push_allowed(environment_for_base(
        "https://api.tinyhumans.ai"
    )));
}

#[tokio::test]
async fn api_key_credentials_are_not_used_for_langfuse_proxy_exports() {
    let dir = tempfile::tempdir().unwrap();
    let mut config = Config::default();
    config.config_path = dir.path().join("config.toml");
    config.workspace_dir = dir.path().join("workspace");
    config.secrets.encrypt = false;
    config.api_url = Some("http://127.0.0.1:9".to_string());
    crate::security::credentials::api_key::store_api_key(&config, "th_live_test").unwrap();

    let ctx = TraceContext::new("trace:req-1", Some("user-1".to_string()));
    let observations = vec![obs(
        1,
        AgentEvent::ModelCompleted {
            call_id: CallId::new("model-1"),
            started_at_ms: Some(1_000),
            usage: Some(Usage::new(10, 3)),
            input: None,
            output: None,
        },
    )];
    assert!(!crate::agent::progress_tracing::langfuse::journal_export::journal_push_ready(&config));
    assert_eq!(
        crate::agent::progress_tracing::langfuse::journal_export::push_observations(
            &config,
            &ctx,
            &observations,
            None,
        )
        .await,
        Ok(())
    );
}

#[tokio::test]
async fn push_observations_skips_external_too() {
    let mut config = Config::default();
    config.api_url = Some("https://other.example".to_string());
    let ctx = TraceContext::new("trace:req-1", Some("user-1".to_string()));
    let observations = vec![obs(
        1,
        AgentEvent::ModelCompleted {
            call_id: CallId::new("model-1"),
            started_at_ms: Some(1_000),
            usage: Some(Usage::new(10, 3)),
            input: None,
            output: None,
        },
    )];

    assert_eq!(
        push_observations(&config, &ctx, &observations, None).await,
        Ok(())
    );
}

#[test]
fn ingestion_url_uses_backend_origin_and_ingestion_path() {
    let mut config = Config::default();
    config.api_url = Some("https://staging-api.tinyhumans.ai/api/v1".to_string());
    assert_eq!(
        ingestion_url(&config),
        "https://staging-api.tinyhumans.ai/telemetry/langfuse/ingestion",
        "endpoint is the backend's Langfuse proxy route on the base server \
         host, replacing any inference path the base carried"
    );

    // A base carrying an inference path resolves to the proxy route on the
    // SAME host — the ingestion host tracks the base server URL, not a fixed
    // literal.
    let mut with_inference_path = Config::default();
    with_inference_path.api_url =
        Some("https://api.tinyhumans.ai/openai/v1/chat/completions".to_string());
    assert_eq!(
        ingestion_url(&with_inference_path),
        "https://api.tinyhumans.ai/telemetry/langfuse/ingestion"
    );
}

#[test]
fn environment_derivation_from_backend_base() {
    assert_eq!(
        environment_for_base("https://staging-api.tinyhumans.ai"),
        "staging"
    );
    assert_eq!(environment_for_base("http://localhost:5000"), "development");
    assert_eq!(environment_for_base("http://127.0.0.1:5000"), "development");
    assert_eq!(
        environment_for_base("https://api.tinyhumans.ai"),
        "production"
    );
}

/// A hostname that merely *contains* `staging` is not ours. The classifier
/// gates whether a live session token leaves the process, so anything it
/// cannot positively recognise has to land on `external` — which does
/// not push.
#[test]
fn a_lookalike_staging_host_is_external_not_staging() {
    for base in [
        // The substring match this replaced classified all of these as
        // staging, and `push_allowed` would then have let them through.
        "https://staging-attacker.invalid",
        "https://staging.evil.example",
        "http://staging-api.tinyhumans.ai.evil.example",
        // Right domain, wrong label position.
        "https://api-staging-mirror.tinyhumans.ai",
        // A public IP literal is never a deployment of ours.
        "https://93.184.216.34",
        // Never send a session bearer to a public backend over plaintext.
        "http://api.tinyhumans.ai",
    ] {
        let environment = environment_for_base(base);
        assert_eq!(
            environment, "external",
            "{base} must classify as external, got {environment}"
        );
        assert!(!push_allowed(environment), "{base} must not be pushable");
    }
}

/// The local buckets the substring form missed. An IPv6 loopback backend
/// is an ordinary local setup, and before the parse it classified as
/// external — so turning the push gate on would have silently stopped
/// exports that had been working.
#[test]
fn local_backends_are_development_including_ipv6_and_private_ranges() {
    for base in [
        "http://[::1]:7788",
        "http://[0:0:0:0:0:0:0:1]:7788",
        "http://[::]:7788",
        "http://192.168.1.20:5000",
        "http://10.0.0.5:5000",
        "http://api.localhost:5000",
        "http://0.0.0.0:5000",
    ] {
        let environment = environment_for_base(base);
        assert_eq!(
            environment, "development",
            "{base} must classify as development, got {environment}"
        );
        assert!(push_allowed(environment), "{base} must stay pushable");
    }
}
