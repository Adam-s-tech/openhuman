//! Core `DirectComposioClient` struct and the loopback/HTTPS URL-safety
//! helpers shared by every direct-mode Composio request path.

pub(super) const COMPOSIO_API_BASE_V3: &str = "https://backend.composio.dev/api/v3";

pub(super) fn ensure_https(url: &str) -> anyhow::Result<()> {
    if !url.starts_with("https://") {
        anyhow::bail!(
            "Refusing to transmit sensitive data over non-HTTPS URL: URL scheme must be https"
        );
    }
    Ok(())
}

pub(super) fn is_loopback_http_url(url: &str) -> bool {
    // Parse rather than prefix-match: a raw `starts_with("http://127.0.0.1:")`
    // is fooled by userinfo smuggling like
    // `http://127.0.0.1:8080@evil.com/api/v3/tools`, which reqwest routes to the
    // *parsed* host (`evil.com`). Verify the actual scheme + host and reject any
    // embedded credentials so the insecure-loopback path can never leak the
    // `x-api-key` header to a non-loopback host.
    let Ok(parsed) = url::Url::parse(url) else {
        return false;
    };
    if parsed.scheme() != "http" {
        return false;
    }
    if !parsed.username().is_empty() || parsed.password().is_some() {
        return false;
    }
    match parsed.host() {
        Some(url::Host::Domain(host)) => host.eq_ignore_ascii_case("localhost"),
        Some(url::Host::Ipv4(ip)) => ip.is_loopback(),
        Some(url::Host::Ipv6(ip)) => ip.is_loopback(),
        None => false,
    }
}

pub(super) fn is_loopback_http_base(url: &str) -> bool {
    is_loopback_http_url(&format!("{}/", url.trim_end_matches('/')))
}

/// Slim host-side HTTP reader for Composio's own v3 API with the user's key.
///
/// Composio execution, the OAuth handoff and connection management all run in
/// the `tinyconnectors` module. This client only serves the two reads whose
/// request parameters the pinned module route does not carry
/// (`/connected_accounts?limit=200`, `/tools?toolkit_versions=latest`) and the
/// pre-store API-key probe.
pub struct DirectComposioClient {
    pub(super) api_key: String,
    /// Base URL for Composio v3 endpoints (`{base}/tools`). Production
    /// always uses [`COMPOSIO_API_BASE_V3`] via [`DirectComposioClient::new`];
    /// the `#[cfg(test)]` `new_with_v3_base` constructor lets unit tests point
    /// the direct-mode `/tools` listing at a local axum mock.
    pub(super) base_v3: String,
    pub(super) allow_insecure_loopback: bool,
}
