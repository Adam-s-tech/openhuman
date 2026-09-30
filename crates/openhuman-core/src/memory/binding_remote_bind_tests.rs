//! Binding a remote (`External`) engine: construction, fallback, retry and rebind.

use super::*;

#[cfg(feature = "memory-remote")]
mod remote_binding {
    use super::*;

    fn remote_cfg(id: &str, endpoint: &str) -> MemorySubsystemConfig {
        let mut cfg = MemorySubsystemConfig {
            driver: id.into(),
            ..Default::default()
        };
        cfg.drivers.insert(
            id.into(),
            MemoryDriverConfig {
                class: Some("external".into()),
                transport: Some("http".into()),
                endpoint: Some(endpoint.into()),
                credential_ref: None,
                trust_state: "trusted".into(),
                deployment: None,
            },
        );
        cfg
    }

    #[tokio::test]
    async fn a_trusted_remote_engine_binds_as_external() {
        let dir = tempfile::tempdir().unwrap();
        let cfg = remote_cfg("supermemory", "https://api.supermemory.ai");
        let binding = for_workspace(dir.path(), &cfg).expect("binding resolves");
        assert_eq!(binding.driver_id(), "supermemory");
        assert_eq!(binding.class(), DriverClass::External);
        assert!(binding.fallback().is_none(), "{:?}", binding.fallback());
    }

    #[tokio::test]
    async fn the_hosted_engine_binds_only_when_a_backend_transport_exists() {
        let dir = tempfile::tempdir().unwrap();
        let cfg = MemorySubsystemConfig {
            driver: "tinyhumans".into(),
            ..Default::default()
        };
        let binding = for_workspace(dir.path(), &cfg).expect("binding resolves");
        // Whether a transport is installed is process-global (other suites
        // install one), so both outcomes are legitimate; what must hold is
        // that each is coherent and never half-built.
        match binding.fallback() {
            None => {
                assert_eq!(binding.driver_id(), "tinyhumans");
                assert_eq!(binding.class(), DriverClass::External);
            }
            Some(fallback) => {
                assert_eq!(binding.driver_id(), "null");
                assert_eq!(fallback.configured_driver, "tinyhumans");
                assert!(
                    fallback.reason.contains("BACKEND_UNAVAILABLE"),
                    "{}",
                    fallback.reason
                );
            }
        }
    }

    #[tokio::test]
    async fn a_failed_engine_build_never_leaks_the_endpoint() {
        let dir = tempfile::tempdir().unwrap();
        let mut cfg = remote_cfg("mem0", "https://leaky-host.example");
        cfg.drivers.get_mut("mem0").unwrap().deployment = Some("bogus".into());
        let binding = for_workspace(dir.path(), &cfg).expect("binding resolves");
        let fallback = binding.fallback().expect("a bad deployment falls back");
        assert!(
            !fallback.reason.contains("leaky-host"),
            "endpoint leaked into an operator-facing string: {}",
            fallback.reason
        );
    }

    #[tokio::test]
    async fn rebind_evicts_the_external_binding_and_binds_the_new_config() {
        let dir = tempfile::tempdir().unwrap();
        let ext = remote_cfg("supermemory", "https://api.supermemory.ai");
        let first = for_workspace(dir.path(), &ext).unwrap();
        assert_eq!(first.class(), DriverClass::External);

        let null_cfg = MemorySubsystemConfig {
            driver: "null".into(),
            ..Default::default()
        };
        let after = rebind(dir.path(), "supermemory", &null_cfg).unwrap();
        assert_eq!(after.driver_id(), "null");
        // The old external binding is gone from the cache: resolving its
        // config again builds a fresh one instead of returning the old Arc.
        let again = for_workspace(dir.path(), &ext).unwrap();
        assert!(!Arc::ptr_eq(&first, &again));
    }
}

#[cfg(feature = "memory-remote")]
mod transient_bind {
    use super::*;

    fn bad_credential_cfg() -> MemorySubsystemConfig {
        let mut cfg = MemorySubsystemConfig {
            driver: "supermemory".into(),
            ..Default::default()
        };
        cfg.drivers.insert(
            "supermemory".into(),
            MemoryDriverConfig {
                class: Some("external".into()),
                transport: Some("http".into()),
                endpoint: Some("https://api.supermemory.ai".into()),
                // Unparseable reference: the driver is admitted but cannot be built.
                credential_ref: Some("not-a-reference".into()),
                trust_state: "trusted".into(),
                deployment: None,
            },
        );
        cfg
    }

    #[tokio::test]
    async fn a_construction_failure_of_an_external_driver_falls_back_to_null_and_retries() {
        let dir = tempfile::tempdir().unwrap();
        let cfg = bad_credential_cfg();
        let first = for_workspace(dir.path(), &cfg).unwrap();
        assert_eq!(
            first.driver_id(),
            "null",
            "never the module: that would split the data"
        );
        assert_eq!(first.fallback().unwrap().configured_driver, "supermemory");
        assert!(
            first.retry_at.is_some(),
            "a construction failure is transient"
        );
        assert!(!first.retry_due());

        // Within the backoff the cached fallback is served (no hammering).
        let again = for_workspace(dir.path(), &cfg).unwrap();
        assert!(Arc::ptr_eq(&first, &again));

        // Once the backoff has passed, the next resolve retries the bind.
        let key = (dir.path().to_path_buf(), "memory".to_string(), cfg.clone());
        let expired = Arc::new(
            crate::memory::binding_build::build(dir.path(), "memory", &cfg)
                .retry_after(std::time::Duration::ZERO),
        );
        BINDINGS
            .get()
            .unwrap()
            .write()
            .unwrap()
            .insert(key, Arc::clone(&expired));
        assert!(expired.retry_due());
        let retried = for_workspace(dir.path(), &cfg).unwrap();
        assert!(
            !Arc::ptr_eq(&expired, &retried),
            "an expired fallback is rebuilt"
        );
        let cached = for_workspace(dir.path(), &cfg).unwrap();
        assert!(
            Arc::ptr_eq(&retried, &cached),
            "the retry result is cached again"
        );
    }

    #[tokio::test]
    async fn an_admission_refusal_is_not_transient() {
        let dir = tempfile::tempdir().unwrap();
        let mut cfg = bad_credential_cfg();
        cfg.drivers.get_mut("supermemory").unwrap().trust_state = "untrusted".into();
        let binding = for_workspace(dir.path(), &cfg).unwrap();
        assert!(binding.fallback().is_some());
        assert!(
            binding.retry_at.is_none(),
            "a deterministic refusal stays cached"
        );
    }
}
