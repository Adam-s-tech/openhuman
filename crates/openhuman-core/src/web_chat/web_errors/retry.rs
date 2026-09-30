//! Parsing and formatting the "retry after" hint upstream providers supply,
//! and telling a retryable 429 apart from a non-retryable business one.

/// Extract a Retry-After / retry_after seconds hint from a free-form
/// error string. Mirrors the typed [`crate::inference::
/// provider::error_classify::parse_retry_after_ms`] helper but operates on
/// the already-flattened `String` that reaches the channel-classifier
/// layer.
///
/// Returns `Some(n)` when a non-negative integer or fractional value
/// follows one of the canonical headers; fractional values are
/// rounded up so the user is never told to retry sooner than the
/// upstream actually allows.
pub(crate) fn parse_retry_after_secs_from_str(err: &str) -> Option<u64> {
    // Normalise quoted JSON-key wrappers ("retry_after": 30) by
    // stripping double quotes before scanning for prefixes
    // (CodeRabbit review on #2371). A serialised provider body like
    // `{"retry_after": 30}` would otherwise miss every prefix and
    // the user would lose the retry hint the provider supplied.
    let normalized = err.to_ascii_lowercase().replace('"', "");
    for prefix in &[
        "retry-after:",
        "retry_after:",
        "retry-after ",
        "retry_after ",
        // Managed backend (#870) emits the structured `retryAfter` field
        // (camelCase). After lower-casing + quote-stripping above it
        // collapses to `retryafter: 30` / `retryafter 30`, so the
        // separator-bearing prefixes here let the same parser surface the
        // structured field the spec asks us to prefer (F5).
        "retryafter:",
        "retryafter ",
    ] {
        if let Some(pos) = normalized.find(prefix) {
            let after = &normalized[pos + prefix.len()..];
            let num_str: String = after
                .trim()
                .chars()
                .take_while(|c| c.is_ascii_digit() || *c == '.')
                .collect();
            if let Ok(secs) = num_str.parse::<f64>() {
                if secs.is_finite() && secs >= 0.0 {
                    return Some(secs.ceil() as u64);
                }
            }
        }
    }
    None
}

/// Format the retry-after hint as a short user-friendly suffix
/// (`" Try again in 30 seconds."`). Returns an empty string when no
/// hint is available so callers can `format!("{summary}{hint}")`
/// without branching on `Option`.
pub(crate) fn retry_after_hint(secs: Option<u64>) -> String {
    match secs {
        Some(0) => " You can retry immediately.".to_string(),
        Some(1) => " Try again in 1 second.".to_string(),
        Some(n) if n < 90 => format!(" Try again in {n} seconds."),
        Some(n) => {
            // Round UP — never tell the user to retry sooner than
            // the upstream actually allows. 90–119s used to render
            // as "about 1 minutes" both because of integer flooring
            // and missing singular/plural handling (CodeRabbit
            // review on #2371).
            let mins = (n / 60) + u64::from(n % 60 != 0);
            let unit = if mins == 1 { "minute" } else { "minutes" };
            format!(" Try again in about {mins} {unit}.")
        }
        None => String::new(),
    }
}

/// Whether a flattened, already-lowercased 429 message is a business limit
/// (plan, balance, quota, package) that a retry cannot clear.
///
/// The reliable provider classifies 429s into retryable vs non-retryable from
/// the typed error, but that `anyhow::Error` is collapsed to a `String` at the
/// native-bus boundary before reaching this layer. The verdict is the one
/// `classify_provider_failure` reaches; the marker list and the Z.AI business
/// codes (1113 / 1311) live in `tinyinference_llm::failure`, not here.
///
/// Caller passes the already-lowercased error string to avoid double
/// allocation.
pub(crate) fn is_non_retryable_rate_limit_text(lower: &str) -> bool {
    tinyinference_llm::failure::contains_business_limit(lower)
}
