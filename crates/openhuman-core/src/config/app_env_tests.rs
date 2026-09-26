use super::*;

#[test]
fn is_staging_app_env_matches_staging_case_insensitively() {
    assert!(is_staging_app_env(Some("staging")));
    assert!(is_staging_app_env(Some(" STAGING ")));
    assert!(!is_staging_app_env(Some("production")));
    assert!(!is_staging_app_env(None));
}
