//! Shared URL validation + SSRF guards for outbound network tools; the
//! implementation lives in [`tinytools_std::url_guard`].

pub use tinytools_std::url_guard::{
    extract_host, extract_port, host_matches_allowlist, is_non_global_v4, is_non_global_v6,
    is_private_or_local_host, normalize_allowed_domains, normalize_domain, validate_url,
    validate_url_with_dns_check,
};
