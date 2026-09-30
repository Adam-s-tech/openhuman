use super::*;

#[test]
fn credential_ref_names_the_engine_specific_keychain_entry() {
    assert_eq!(
        credential_ref_for("supermemory"),
        "keychain:memory-supermemory"
    );
    let parsed = crate::security::credentials::credential_ref::CredentialRef::parse(
        &credential_ref_for("mem0"),
    )
    .unwrap();
    assert_eq!(parsed.name(), "memory-mem0");
}

#[test]
fn keyring_partition_follows_the_profile_dir() {
    let user = keyring_user_id(Path::new("/home/u/.openhuman/users/abc/workspace"));
    assert_eq!(user, "abc");
    assert_ne!(
        keyring_user_id(Path::new("/a/one/workspace")),
        keyring_user_id(Path::new("/a/two/workspace")),
        "two profiles must not share an engine key"
    );
    assert!(keyring_user_id(Path::new("workspace")).starts_with("memory-path-"));
}

#[test]
fn scrub_removes_every_secret_and_ignores_blank_ones() {
    let scrubbed = scrub(
        "failed to reach https://leak.example with key sk-123",
        &["https://leak.example", "sk-123", "", "  "],
    );
    assert!(
        !scrubbed.contains("leak.example") && !scrubbed.contains("sk-123"),
        "{scrubbed}"
    );
}

#[test]
fn engine_target_from_entry_copies_endpoint_deployment_and_ref() {
    let entry = MemoryDriverConfig {
        class: Some("external".into()),
        transport: Some("http".into()),
        endpoint: Some("https://e.example".into()),
        credential_ref: Some("keychain:memory-mem0".into()),
        trust_state: "trusted".into(),
        deployment: Some("cloud".into()),
    };
    let target = EngineTarget::from_entry("mem0", Some(&entry));
    assert_eq!(target.id, "mem0");
    assert_eq!(target.endpoint.as_deref(), Some("https://e.example"));
    assert_eq!(target.deployment.as_deref(), Some("cloud"));
    assert_eq!(
        target.credential_ref.as_deref(),
        Some("keychain:memory-mem0")
    );
    assert!(target.api_key.is_none());
    assert!(EngineTarget::from_entry("mem0", None).endpoint.is_none());
}

#[test]
fn api_url_override_is_read_from_the_workspace_config() {
    let dir = tempfile::tempdir().unwrap();
    let workspace = dir.path().join("workspace");
    std::fs::create_dir_all(&workspace).unwrap();
    assert_eq!(
        configured_api_url(&workspace),
        None,
        "no config, no override"
    );
    std::fs::write(
        dir.path().join("config.toml"),
        "api_url = \" https://staging.example \"\n",
    )
    .unwrap();
    assert_eq!(
        configured_api_url(&workspace).as_deref(),
        Some("https://staging.example")
    );
    std::fs::write(dir.path().join("config.toml"), "api_url = \"\"\n").unwrap();
    assert_eq!(
        configured_api_url(&workspace),
        None,
        "blank means no override"
    );
}

#[cfg(feature = "memory-remote")]
#[test]
fn only_factory_engines_are_remote() {
    assert!(is_remote_engine("supermemory"));
    assert!(is_remote_engine("tinyhumans"));
    assert!(!is_remote_engine("tinymemory"));
    assert!(!is_remote_engine("null"));
    assert!(!is_remote_engine("bogus"));
}

#[cfg(feature = "memory-remote")]
#[tokio::test]
async fn the_hosted_bearer_reads_the_live_credential_and_reports_a_missing_session() {
    use tinymemory::factory::BearerSource;
    let dir = tempfile::tempdir().unwrap();
    // No config, no credential: the source must fail with the sentinel the RPC
    // layer maps to SESSION_EXPIRED, never return an empty token.
    let source = LiveSessionBearer::new(&dir.path().join("workspace"));
    let error = source.bearer().await.expect_err("no credential");
    assert!(error.to_string().starts_with("SESSION_EXPIRED:"), "{error}");
}

#[cfg(feature = "memory-remote")]
#[tokio::test]
async fn the_hosted_bearer_is_cached_for_its_ttl_and_failures_are_not() {
    use tinymemory::factory::BearerSource;
    let dir = tempfile::tempdir().unwrap();
    let workspace = dir.path().join("workspace");
    std::fs::create_dir_all(&workspace).unwrap();
    std::fs::write(
        dir.path().join("config.toml"),
        "[secrets]\nencrypt = false\n",
    )
    .unwrap();
    let config = crate::config::ops::load_config_for_workspace_with_timeout(&workspace)
        .await
        .expect("load config");

    let cached = LiveSessionBearer::with_ttl(&workspace, std::time::Duration::from_secs(60));
    // No credential yet: an error, and the error is not cached.
    assert!(cached.bearer().await.is_err());
    crate::security::credentials::api_key::store_api_key(&config, "tiny_live_first").unwrap();
    assert_eq!(cached.bearer().await.unwrap(), "tiny_live_first");

    // Removing the credential does not disturb a cached bearer within the TTL...
    crate::security::credentials::api_key::clear_api_key(&config).unwrap();
    assert_eq!(cached.bearer().await.unwrap(), "tiny_live_first");
    // ...but a source with no cache window sees the change immediately.
    let uncached = LiveSessionBearer::with_ttl(&workspace, std::time::Duration::ZERO);
    assert!(uncached.bearer().await.is_err());
}
