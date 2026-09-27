use super::*;
use crate::config::schema::tools::http::HttpRequestConfig;

fn legacy(toml_body: &str) -> SearchConfig {
    toml::from_str(toml_body).expect("legacy search section parses")
}

fn provider(cfg: &SearchConfig, name: &str) -> Option<SearchProviderSettings> {
    cfg.providers.get(name).copied()
}

#[test]
fn fresh_defaults_are_managed_exa_and_gemini_with_roles_presentation() {
    let cfg = SearchConfig::default();
    assert!(!cfg.needs_migration());
    assert!(cfg.is_enabled());
    assert_eq!(cfg.presentation, SearchPresentation::Roles);
    assert_eq!(provider(&cfg, "exa"), Some(SearchProviderSettings::managed()));
    assert_eq!(provider(&cfg, "gemini"), Some(SearchProviderSettings::managed()));
    assert_eq!(cfg.providers.len(), 2);
    assert!(cfg.roles.is_empty());
}

#[test]
fn http_request_defaults_to_allow_all() {
    // Web research works out of the box: the default allowlist is the
    // wildcard. The SSRF guard (url_guard) still blocks local/private
    // hosts regardless, so this only opens public sites.
    let cfg = HttpRequestConfig::default();
    assert_eq!(cfg.allowed_domains, vec!["*".to_string()]);
    assert_eq!(cfg.max_response_size, 1_000_000);
    assert_eq!(cfg.timeout_secs, 30);
}

#[test]
fn a_file_without_schema_version_needs_migration() {
    let cfg = legacy("engine = \"managed\"\n");
    assert!(cfg.needs_migration());
    assert!(cfg.providers.is_empty());
}

#[test]
fn legacy_managed_becomes_managed_exa_and_gemini() {
    let mut cfg = legacy("engine = \"managed\"\n");
    assert!(cfg.migrate_legacy(LegacySearchInputs::default()));
    assert!(cfg.is_enabled());
    assert_eq!(provider(&cfg, "exa"), Some(SearchProviderSettings::managed()));
    assert_eq!(provider(&cfg, "gemini"), Some(SearchProviderSettings::managed()));
    assert!(cfg.roles.is_empty());
    assert_eq!(cfg.schema_version, SEARCH_SCHEMA_VERSION);
    assert!(cfg.engine.is_none());
}

#[test]
fn migration_is_idempotent() {
    let mut cfg = legacy("engine = \"brave\"\n[brave]\napi_key = \"k\"\n");
    assert!(cfg.migrate_legacy(LegacySearchInputs::default()));
    let snapshot = format!("{cfg:?}");
    assert!(!cfg.migrate_legacy(LegacySearchInputs {
        tinyfish_active: true,
        ..Default::default()
    }));
    assert_eq!(format!("{cfg:?}"), snapshot);
}

#[test]
fn legacy_disabled_stays_disabled_but_keeps_providers_configured() {
    let mut cfg = legacy("engine = \"disabled\"\n");
    cfg.migrate_legacy(LegacySearchInputs::default());
    assert!(!cfg.is_enabled());
    assert!(cfg.providers.contains_key("exa"));
}

#[test]
fn legacy_byok_engine_leads_the_search_role() {
    let mut cfg = legacy("engine = \"tavily\"\n[tavily]\napi_key = \"t\"\n");
    cfg.migrate_legacy(LegacySearchInputs::default());
    assert_eq!(provider(&cfg, "tavily"), Some(SearchProviderSettings::direct()));
    assert_eq!(
        cfg.roles.get(SEARCH_ROLE_SEARCH),
        Some(&vec!["tavily".to_string(), "exa".to_string()])
    );
}

#[test]
fn legacy_byok_engine_without_key_does_not_claim_a_role() {
    let mut cfg = legacy("engine = \"brave\"\n");
    cfg.migrate_legacy(LegacySearchInputs::default());
    assert!(!cfg.providers.contains_key("brave"));
    assert!(cfg.roles.is_empty());
}

#[test]
fn legacy_exa_engine_with_key_keeps_exa_direct() {
    let mut cfg = legacy("engine = \"exa\"\n[exa]\napi_key = \"e\"\n");
    cfg.migrate_legacy(LegacySearchInputs::default());
    assert_eq!(provider(&cfg, "exa"), Some(SearchProviderSettings::direct()));
    assert_eq!(cfg.exa.key(), Some("e"));
}

#[test]
fn exa_key_under_managed_engine_stays_managed() {
    let mut cfg = legacy("engine = \"managed\"\n[exa]\napi_key = \"e\"\n");
    cfg.migrate_legacy(LegacySearchInputs::default());
    assert_eq!(provider(&cfg, "exa"), Some(SearchProviderSettings::managed()));
}

#[test]
fn parallel_selection_and_key_are_dropped() {
    let mut cfg = legacy("engine = \"parallel\"\n[parallel]\napi_key = \"p\"\n");
    cfg.migrate_legacy(LegacySearchInputs::default());
    assert!(!cfg.providers.contains_key("parallel"));
    assert!(cfg.parallel.key().is_none());
    // Parallel's roles fall to the managed defaults.
    assert_eq!(provider(&cfg, "exa"), Some(SearchProviderSettings::managed()));
    let written = toml::to_string(&cfg).unwrap();
    assert!(!written.contains("parallel"), "{written}");
    assert!(!written.contains("engine"), "{written}");
}

#[test]
fn explicit_provider_selection_maps_managed_and_routes() {
    let mut cfg = legacy(
        "enabled = true\nenabled_providers = [\"managed\", \"parallel\", \"gemini\", \"tinyfish\", \"querit\"]\n\
         gemini_route = \"direct\"\n[gemini]\napi_key = \"g\"\n",
    );
    cfg.migrate_legacy(LegacySearchInputs::default());
    assert_eq!(provider(&cfg, "exa"), Some(SearchProviderSettings::managed()));
    assert_eq!(provider(&cfg, "gemini"), Some(SearchProviderSettings::direct()));
    assert_eq!(provider(&cfg, "tinyfish"), Some(SearchProviderSettings::managed()));
    assert_eq!(provider(&cfg, "querit"), Some(SearchProviderSettings::direct()));
    assert!(!cfg.providers.contains_key("parallel"));
    assert!(cfg.enabled_providers.is_none());
}

#[test]
fn legacy_toggles_outside_search_migrate_when_active() {
    let mut cfg = legacy("engine = \"managed\"\n");
    cfg.migrate_legacy(LegacySearchInputs {
        tinyfish_active: true,
        seltz_active: true,
        searxng_active: true,
    });
    assert_eq!(provider(&cfg, "tinyfish"), Some(SearchProviderSettings::managed()));
    assert_eq!(provider(&cfg, "seltz"), Some(SearchProviderSettings::direct()));
    assert_eq!(provider(&cfg, "searxng"), Some(SearchProviderSettings::direct()));
}

#[test]
fn managed_presentation_provider_is_cleared() {
    let mut cfg = legacy("presentation = \"router\"\npresentation_provider = \"managed\"\n");
    cfg.migrate_legacy(LegacySearchInputs::default());
    assert_eq!(cfg.presentation, SearchPresentation::Router);
    assert!(cfg.presentation_provider.is_none());
}

#[test]
fn route_is_direct_for_providers_that_cannot_be_managed() {
    let mut cfg = SearchConfig::default();
    cfg.providers
        .insert("brave".into(), SearchProviderSettings::managed());
    assert_eq!(cfg.route("brave"), SearchRoute::Direct);
    assert_eq!(cfg.route("exa"), SearchRoute::Managed);
    assert_eq!(cfg.route("unknown"), SearchRoute::Direct);
}

#[test]
fn enabled_provider_names_skip_disabled_entries() {
    let mut cfg = SearchConfig::default();
    cfg.providers.insert(
        "tavily".into(),
        SearchProviderSettings {
            enabled: false,
            route: SearchRoute::Direct,
        },
    );
    let names: Vec<_> = cfg.enabled_provider_names().into_iter().collect();
    assert_eq!(names, vec!["exa".to_string(), "gemini".to_string()]);
}

#[test]
fn route_and_presentation_parse_accept_legacy_spellings() {
    assert_eq!(SearchRoute::parse("backend"), Some(SearchRoute::Managed));
    assert_eq!(SearchRoute::parse(" Direct "), Some(SearchRoute::Direct));
    assert_eq!(SearchRoute::parse("other"), None);
    assert_eq!(
        SearchPresentation::parse("all_tools"),
        Some(SearchPresentation::AllTools)
    );
    assert_eq!(SearchPresentation::parse("nope"), None);
}

#[test]
fn credentials_resolve_per_provider() {
    let mut cfg = SearchConfig::default();
    cfg.credentials_mut("gemini").unwrap().api_key = Some(" g ".into());
    assert_eq!(cfg.credentials("gemini_deep_research").unwrap().key(), Some("g"));
    assert!(cfg.credentials("seltz").is_none());
}
