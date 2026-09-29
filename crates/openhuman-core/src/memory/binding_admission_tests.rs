use super::*;

#[test]
fn admit_default_config_binds_module_tinymemory() {
    let (id, class) = admit(&MemorySubsystemConfig::default()).expect("default config admits");
    assert_eq!(id, "tinymemory");
    assert_eq!(class, DriverClass::Module);
}

#[test]
fn admit_null_driver_binds_null_class() {
    let cfg = MemorySubsystemConfig {
        driver: "null".into(),
        ..Default::default()
    };
    let (id, class) = admit(&cfg).expect("null driver admits");
    assert_eq!(id, "null");
    assert_eq!(class, DriverClass::Null);
}

#[test]
fn admit_builtin_module_driver_id_gets_module_class() {
    // Regression for the reviewer finding: before this, any non-null id without
    // a drivers entry — a typo like "tinycortx", or an external backend that
    // forgot its table — was silently classified Embedded. Only the two built-in
    // ids admit implicitly.
    let cfg = MemorySubsystemConfig {
        driver: "tinymemory".into(),
        ..Default::default()
    };
    let (id, class) = admit(&cfg).expect("the module default id admits");
    assert_eq!(id, "tinymemory");
    assert_eq!(class, DriverClass::Module);
}

#[test]
fn admit_refuses_an_unregistered_non_null_driver_id() {
    // A typo or an external backend with no `drivers.<id>` entry must not
    // silently run the module under an invented driver id.
    let cfg = MemorySubsystemConfig {
        driver: "supermemory".into(),
        ..Default::default()
    };
    let refusal = admit(&cfg).expect_err("an unregistered id must be refused");
    assert_eq!(refusal.configured_driver, "supermemory");
    assert!(
        refusal.reason.contains("supermemory"),
        "refusal must name the offending id: {}",
        refusal.reason
    );
    assert!(
        refusal.reason.contains("drivers"),
        "refusal must point at the missing drivers table: {}",
        refusal.reason
    );
}

#[test]
fn admit_refuses_non_builtin_id_even_with_a_drivers_entry_that_says_no_class() {
    // Same rule when an entry exists but carries no `class` line: only the two
    // built-in ids imply a class. An arbitrary id must not silently become
    // Embedded just because someone registered a placeholder entry.
    let mut cfg = MemorySubsystemConfig {
        driver: "custom-mem".into(),
        ..Default::default()
    };
    cfg.drivers.insert(
        "custom-mem".into(),
        MemoryDriverConfig {
            class: None,
            ..Default::default()
        },
    );
    let refusal = admit(&cfg).expect_err("entry with no class must not admit an arbitrary id");
    assert_eq!(refusal.configured_driver, "custom-mem");
    assert!(
        refusal.reason.contains("custom-mem"),
        "refusal must name the offending id: {}",
        refusal.reason
    );
    assert!(
        refusal.reason.contains("class line"),
        "refusal must point at the missing class line: {}",
        refusal.reason
    );
}

#[test]
fn admit_refuses_an_unregistered_module_id() {
    let mut cfg = MemorySubsystemConfig {
        driver: "custom-mem".into(),
        ..Default::default()
    };
    cfg.drivers.insert(
        "custom-mem".into(),
        MemoryDriverConfig {
            class: Some("module".into()),
            ..Default::default()
        },
    );
    let refusal = admit(&cfg).expect_err("only a registered TinyBus module may bind");
    assert!(refusal.reason.contains("not registered"));
}

#[test]
fn admit_refuses_the_removed_embedded_class() {
    let refusal = admit(&cfg_with_class("custom-mem", "embedded"))
        .expect_err("the in-process memory engine was removed");
    assert!(refusal.reason.contains("no longer supported"));
}

#[test]
fn admit_refuses_untrusted_external_driver() {
    // The default trust_state is "untrusted" (kernel.md §3.4, fail-closed).
    let cfg = external_driver_cfg(&MemoryDriverConfig::default().trust_state);
    let refusal = admit(&cfg).expect_err("untrusted external driver must be refused");
    assert_eq!(refusal.configured_driver, "supermemory");
    assert!(
        refusal.reason.contains("trust_state"),
        "refusal must name the trust rule: {}",
        refusal.reason
    );
}

#[cfg(not(feature = "memory-remote"))]
#[test]
fn admit_refuses_trusted_external_driver_when_memory_remote_is_off() {
    let cfg = external_driver_cfg("trusted");
    let refusal = admit(&cfg).expect_err("no external transport without memory-remote");
    assert!(
        refusal.reason.contains("transport"),
        "refusal must name the missing transport: {}",
        refusal.reason
    );
    assert!(
        !refusal.reason.contains("trust_state"),
        "a trusted driver must not be refused for trust: {}",
        refusal.reason
    );
}

#[cfg(not(feature = "memory-remote"))]
#[test]
fn admit_refuses_the_hosted_engine_when_memory_remote_is_off() {
    let cfg = MemorySubsystemConfig {
        driver: "tinyhumans".into(),
        ..Default::default()
    };
    let refusal = admit(&cfg).expect_err("no hosted engine without memory-remote");
    assert!(
        refusal.reason.contains("not built in"),
        "{}",
        refusal.reason
    );
}

#[cfg(feature = "memory-remote")]
#[test]
fn admit_admits_a_trusted_external_engine_the_factory_knows() {
    let (id, class) = admit(&external_driver_cfg("trusted")).expect("trusted supermemory admits");
    assert_eq!(id, "supermemory");
    assert_eq!(class, DriverClass::External);
}

#[cfg(feature = "memory-remote")]
#[test]
fn admit_refuses_a_trusted_external_id_the_factory_does_not_know() {
    let mut cfg = external_driver_cfg("trusted");
    cfg.driver = "not-an-engine".into();
    let entry = cfg.drivers.remove("supermemory").unwrap();
    cfg.drivers.insert("not-an-engine".into(), entry);
    let refusal = admit(&cfg).expect_err("an id the factory cannot build must be refused");
    assert!(
        refusal.reason.contains("not an engine this build can bind"),
        "{}",
        refusal.reason
    );
    assert!(
        !refusal.reason.contains("trust_state"),
        "{}",
        refusal.reason
    );
}

#[cfg(feature = "memory-remote")]
#[test]
fn admit_admits_the_hosted_engine_without_a_drivers_entry() {
    let cfg = MemorySubsystemConfig {
        driver: "tinyhumans".into(),
        ..Default::default()
    };
    let (id, class) = admit(&cfg).expect("tinyhumans is first-party");
    assert_eq!(id, "tinyhumans");
    assert_eq!(class, DriverClass::External);
}

#[cfg(feature = "memory-remote")]
#[test]
fn admit_trusts_the_hosted_engine_implicitly_even_with_an_untrusted_entry() {
    let mut cfg = external_driver_cfg("untrusted");
    cfg.driver = "tinyhumans".into();
    let entry = cfg.drivers.remove("supermemory").unwrap();
    cfg.drivers.insert("tinyhumans".into(), entry);
    let (_, class) = admit(&cfg).expect("first-party engines need no trust grant");
    assert_eq!(class, DriverClass::External);
}

#[cfg(feature = "memory-remote")]
#[test]
fn untrusted_external_engines_stay_refused_with_memory_remote_on() {
    let refusal = admit(&external_driver_cfg("untrusted")).expect_err("fail closed");
    assert!(refusal.reason.contains("trust_state"), "{}", refusal.reason);
}

#[test]
fn admit_rejects_an_unknown_driver_class() {
    let mut cfg = external_driver_cfg("trusted");
    cfg.drivers.get_mut("supermemory").unwrap().class = Some("embeded".into());
    let refusal = admit(&cfg).expect_err("typo'd class must be refused");
    assert!(
        refusal.reason.contains("embeded"),
        "refusal must echo the typo: {}",
        refusal.reason
    );
}

#[test]
fn fallback_reason_never_contains_credential_ref_or_endpoint() {
    let mut cfg = external_driver_cfg("untrusted");
    cfg.drivers.get_mut("supermemory").unwrap().credential_ref =
        Some("keychain:super-secret-value".into());
    let refusal = admit(&cfg).expect_err("untrusted external driver must be refused");
    assert!(
        !refusal.reason.contains("super-secret-value"),
        "credential_ref leaked into an operator-facing string: {}",
        refusal.reason
    );
    assert!(
        !refusal.reason.contains("supermemory.ai"),
        "endpoint leaked into an operator-facing string: {}",
        refusal.reason
    );
}

#[test]
fn for_workspace_caches_binding_per_workspace() {
    let dir_a = tempfile::tempdir().unwrap();
    let dir_b = tempfile::tempdir().unwrap();
    let cfg = MemorySubsystemConfig::default();

    let a = for_workspace(dir_a.path(), &cfg).expect("bind workspace A");
    let b = for_workspace(dir_b.path(), &cfg).expect("bind workspace B");
    assert!(
        !Arc::ptr_eq(&a, &b),
        "different workspaces must get isolated bindings"
    );

    let a_again = for_workspace(dir_a.path(), &cfg).expect("re-resolve workspace A");
    assert!(
        Arc::ptr_eq(&a, &a_again),
        "same workspace must reuse the cached binding"
    );
}

#[test]
fn same_workspace_with_changed_config_binds_fresh() {
    // `CoreContext::rebind_workspace` treats "same workspace, changed
    // [subsystems.memory]" as a real rebind (a changed driver/hooks/trust all
    // feed `build`). The cache must key on the config as well as the path, or
    // a changed config for an already-bound workspace would keep serving the
    // previous driver until process restart.
    let dir = tempfile::tempdir().unwrap();
    let default = MemorySubsystemConfig::default();
    let null = MemorySubsystemConfig {
        driver: "null".into(),
        ..Default::default()
    };

    let tiny = for_workspace(dir.path(), &default).expect("bind tinymemory");
    assert_eq!(tiny.driver_id(), "tinymemory");

    // Same (workspace, config) pair reuses the cached binding...
    let tiny_again = for_workspace(dir.path(), &default).expect("re-bind tinymemory");
    assert!(
        Arc::ptr_eq(&tiny, &tiny_again),
        "unchanged config must reuse the cached binding"
    );

    // ...but a changed config for the SAME workspace must bind fresh.
    let null_binding = for_workspace(dir.path(), &null).expect("bind null");
    assert!(
        !Arc::ptr_eq(&tiny, &null_binding),
        "changed config must bind fresh, not serve the stale tinymemory driver"
    );
    assert_eq!(null_binding.driver_id(), "null");

    // Reverting to the original config still resolves its own binding. This is
    // the transient-mismatch half: a stale (workspace, config) pairing never
    // shadows the correct pair, so it cannot permanently pin a workspace to the
    // wrong driver (the atomicity concern in the login/logout rebind).
    let tiny_reverted = for_workspace(dir.path(), &default).expect("re-bind tinymemory");
    assert!(
        Arc::ptr_eq(&tiny, &tiny_reverted),
        "returning to the original config must serve the original binding"
    );
}

#[test]
fn module_class_binds_the_module_driver_not_null() {
    // Plain `#[test]`: no tokio runtime. Binding must stay synchronous and
    // I/O-free, which is why the module provider resolves its client lazily.
    let dir = tempfile::tempdir().unwrap();
    let workspace = dir.path().join("never-created");
    let binding =
        for_workspace(&workspace, &MemorySubsystemConfig::default()).expect("default bind");

    assert_eq!(binding.driver_id(), "tinymemory");
    assert_eq!(binding.class(), DriverClass::Module);
    assert!(binding.fallback().is_none());
    assert_ne!(binding.unguarded_provider().driver_id(), NULL_DRIVER_ID);
    assert!(binding.capabilities().contains(Capability::Core));
    assert!(binding.capabilities().validate().is_ok());
    assert!(
        !workspace.exists(),
        "binding must not touch the workspace on disk"
    );
}

#[test]
fn module_binding_advertises_the_pinned_artifacts_families() {
    // Was `module_binding_advertises_every_family`, asserting `advertised ==
    // Capabilities::all()`. That encoded #5598 as expected: the host claimed
    // all eighteen contract families while the then-pinned v1.0.1 artifact
    // served thirteen, so the other five (`people`, `chunks`, `retrieval`, `profile`,
    // `episodic`) answered `UnknownMethod` instead of reporting themselves
    // absent. `modules::memory::ARTIFACT_CAPABILITIES` was narrowed to match
    // what the pinned release actually serves (see its module docs); this
    // test now pins the same, corrected boundary instead of the old
    // "bound equals unbound-default" coincidence, which no longer holds.
    let dir = tempfile::tempdir().unwrap();
    let binding =
        for_workspace(dir.path(), &MemorySubsystemConfig::default()).expect("default bind");
    let advertised = binding.capabilities();

    assert!(advertised.contains_all(Capabilities::mandatory()));

    const ARTIFACT_SERVES: [Capability; 13] = [
        Capability::Core,
        Capability::Recall,
        Capability::Ingest,
        Capability::Documents,
        Capability::Tree,
        Capability::Entities,
        Capability::Graph,
        Capability::Diff,
        Capability::Goals,
        Capability::ToolMemory,
        Capability::Sources,
        Capability::Maintenance,
        Capability::Portability,
    ];
    for family in ARTIFACT_SERVES {
        assert!(advertised.contains(family), "{family} must be advertised");
    }

    // Arrived in the v1.2.0 artifact (the first four) and with the Episodic
    // accessor (the fifth — the members were served since v1.2.0, the HOST
    // gained `as_episodic` when the archivist moved onto the family). Asserted
    // PRESENT so a re-pin that silently narrows the advertised set is caught
    // the same way an over-claim would be.
    const SERVED_SINCE_1_2_0: [Capability; 5] = [
        Capability::People,
        Capability::Chunks,
        Capability::Retrieval,
        Capability::Profile,
        Capability::Episodic,
    ];
    for family in SERVED_SINCE_1_2_0 {
        assert!(
            advertised.contains(family),
            "{family} has a bus member in the pinned artifact but is not advertised — \
             the host is under-claiming and hiding a family it can reach"
        );
    }

    // Every contract family is now reachable and advertised; what keeps this
    // honest is the accessor rule (`capabilities_for` can only name families
    // the provider implements) plus the pin-drift test on every registry bump.
    assert_eq!(advertised, Capabilities::all());

    // The contract may be ahead of the artifact but never behind it.
    assert!(Capabilities::all().contains_all(advertised));
    // Unbound contexts still assume the widest set (deny-by-default is wrong
    // pre-boot, per `unbound_default_capabilities`'s own docs) — that is a
    // different, deliberately permissive default, not a claim that a bound
    // module actually serves everything.
    assert_eq!(unbound_default_capabilities(), Capabilities::all());
}

#[test]
fn null_driver_config_still_binds_the_null_provider() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = MemorySubsystemConfig {
        driver: "null".into(),
        ..Default::default()
    };
    let binding = for_workspace(dir.path(), &cfg).expect("null bind");
    assert_eq!(binding.driver_id(), NULL_DRIVER_ID);
    assert_eq!(binding.class(), DriverClass::Null);
    assert_eq!(binding.unguarded_provider().driver_id(), NULL_DRIVER_ID);
    assert!(
        binding.fallback().is_none(),
        "an explicitly requested null driver is not a fallback"
    );
}

#[test]
fn refused_driver_falls_back_to_the_null_placeholder() {
    let dir = tempfile::tempdir().unwrap();
    let binding =
        for_workspace(dir.path(), &external_driver_cfg("untrusted")).expect("bind falls back");
    assert_eq!(binding.driver_id(), "null");
    assert_eq!(binding.class(), DriverClass::Null);
    let fallback = binding.fallback().expect("fallback provenance recorded");
    assert_eq!(fallback.configured_driver, "supermemory");
}

#[test]
fn fallback_binding_advertises_only_mandatory_capabilities() {
    let dir = tempfile::tempdir().unwrap();
    let binding =
        for_workspace(dir.path(), &external_driver_cfg("untrusted")).expect("bind falls back");
    assert_eq!(binding.capabilities(), Capabilities::mandatory());
    // Even the fallback must be a *legal* bind: the mandatory three are present.
    assert!(binding.capabilities().validate().is_ok());
    assert!(!binding.capabilities().contains(Capability::Tree));
}

#[test]
fn unbound_default_is_the_full_capability_set() {
    let all = unbound_default_capabilities();
    assert_eq!(all, Capabilities::all());
    assert_eq!(all.len(), Capability::ALL.len());
}

#[test]
fn bound_driver_view_carries_class_capabilities_and_fallback() {
    let dir = tempfile::tempdir().unwrap();
    let binding =
        for_workspace(dir.path(), &external_driver_cfg("untrusted")).expect("bind falls back");
    let bound = binding.to_bound_driver();
    assert_eq!(bound.slot, SubsystemSlot::Memory);
    assert_eq!(bound.id, "null");
    assert_eq!(bound.class, DriverClass::Null);
    assert_eq!(bound.contract_version, CONTRACT_VERSION);
    assert_eq!(bound.fell_back_from.as_deref(), Some("supermemory"));
    assert!(bound.is_fallback());
    // The generic view carries the same families as opaque strings.
    assert!(bound.capabilities.contains("core"));
    assert!(!bound.capabilities.contains("tree"));
    assert_eq!(bound.capabilities.len(), binding.capabilities().len());
}

#[test]
fn health_converts_as_a_total_three_arm_match() {
    assert_eq!(to_driver_health(MemoryHealth::Ready), DriverHealth::Ready);
    assert_eq!(
        to_driver_health(MemoryHealth::degraded("reindexing")),
        DriverHealth::degraded("reindexing")
    );
    assert_eq!(
        to_driver_health(MemoryHealth::down("refused")),
        DriverHealth::down("refused")
    );
}

#[test]
fn capabilities_are_asked_exactly_once_per_bind() {
    let provider = Arc::new(CountingProvider::new());
    let binding = bind_provider_for_test(provider.clone(), DriverClass::Module);

    for _ in 0..5 {
        assert_eq!(binding.capabilities(), Capabilities::all());
    }
    assert_eq!(binding.driver_id(), "counting");
    assert_eq!(
        provider.calls.load(Ordering::SeqCst),
        1,
        "capabilities() must be asked exactly once, at bind time"
    );
}

#[test]
fn admit_refuses_a_module_class_override_on_the_null_driver() {
    let refusal = admit(&cfg_with_class("null", "module"))
        .expect_err("null must not be re-classed as module");
    assert_eq!(refusal.configured_driver, "null");
    assert!(
        refusal.reason.contains("built in"),
        "refusal must say the id is built in: {}",
        refusal.reason
    );
}

#[test]
fn admit_refuses_a_null_class_override_on_the_module_driver() {
    let refusal = admit(&cfg_with_class(MODULE_ID, "null"))
        .expect_err("tinymemory must not be re-classed as null");
    assert_eq!(refusal.configured_driver, MODULE_ID);
    assert!(
        refusal.reason.contains("built in"),
        "refusal must say the id is built in: {}",
        refusal.reason
    );
}

#[test]
fn admit_accepts_a_class_line_that_agrees_with_the_built_in_id() {
    // Redundant, but not a mistake: confirming the real class is allowed.
    let (id, class) = admit(&cfg_with_class("null", "null")).expect("agreeing class admits");
    assert_eq!(id, "null");
    assert_eq!(class, DriverClass::Null);

    let (id, class) = admit(&cfg_with_class(MODULE_ID, "module")).expect("agreeing class admits");
    assert_eq!(id, MODULE_ID);
    assert_eq!(class, DriverClass::Module);
}

#[test]
fn a_null_class_override_cannot_smuggle_the_module_into_the_binding() {
    // The end-to-end shape of the refusal: `build` must not hand back an
    // module provider for `driver = "null"`.
    let dir = tempfile::tempdir().unwrap();
    let binding = for_workspace(dir.path(), &cfg_with_class("null", "module")).expect("binds");

    assert_eq!(binding.class(), DriverClass::Null);
    assert_eq!(binding.driver_id(), NULL_DRIVER_ID);
    assert!(
        binding.fallback().is_some(),
        "a refused class override must be recorded as a fallback"
    );
}

// ---------------------------------------------------------------------------
// `disables_memory` — deliberate null only
// ---------------------------------------------------------------------------

#[test]
fn an_explicit_null_driver_disables_memory() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = MemorySubsystemConfig {
        driver: "null".into(),
        ..Default::default()
    };
    let binding = for_workspace(dir.path(), &cfg).expect("binds");

    assert!(binding.fallback().is_none(), "this is not a fallback");
    assert!(
        binding.disables_memory(),
        "an operator who bound /dev/null asked for the surface to be gone"
    );
}

#[test]
fn a_fallback_to_null_does_not_disable_memory() {
    // A misconfiguration must be loud, not silently memory-less: the fallback
    // is reported in status and the surface stays present.
    let dir = tempfile::tempdir().unwrap();
    let binding = for_workspace(dir.path(), &external_driver_cfg("untrusted")).expect("binds");

    assert_eq!(binding.class(), DriverClass::Null);
    assert!(binding.fallback().is_some(), "this IS a fallback");
    assert!(!binding.disables_memory());
}

#[test]
fn the_module_driver_never_disables_memory() {
    let dir = tempfile::tempdir().unwrap();
    let binding = for_workspace(dir.path(), &MemorySubsystemConfig::default()).expect("binds");
    assert!(!binding.disables_memory());
}

// A build that admits the module class but cannot construct a module-backed
// provider must not *report* the module class. `bind_provider` receives the
// class that status, `modules.status` and the boot log all read, so passing the
// admitted class while binding a placeholder advertises a live module-backed
// surface with a null store behind it — the one failure this codebase's drift
// guards exist to prevent, and the shape a reviewer caught here.

#[cfg(not(feature = "modules"))]
#[test]
fn a_module_driver_reports_the_null_class_when_the_feature_is_off() {
    let cfg = cfg_with_class("tinymemory", "module");
    let binding = crate::memory::binding_build::build(
        std::path::Path::new("/tmp/openhuman-binding-test"),
        "memory",
        &cfg,
    );
    assert_eq!(
        binding.class(),
        crate::core::subsystem::DriverClass::Null,
        "a placeholder must not be reported as module-backed"
    );
}

#[cfg(feature = "modules")]
#[test]
fn a_module_driver_reports_the_module_class_when_the_feature_is_on() {
    // The other direction, so the arm above cannot be "fixed" by making every
    // module binding report Null. Construction stays I/O-free, so this needs no
    // runtime and loads nothing.
    let cfg = cfg_with_class("tinymemory", "module");
    let binding = crate::memory::binding_build::build(
        std::path::Path::new("/tmp/openhuman-binding-test"),
        "memory",
        &cfg,
    );
    assert_eq!(
        binding.class(),
        crate::core::subsystem::DriverClass::Module,
        "a real module binding must report the module class"
    );
}

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
