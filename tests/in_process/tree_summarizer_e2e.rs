//! E2E coverage for the `tree_summarizer` ingestion buffer.
//!
//! Gate controller `tree_summarizer_ingest` had zero references under `tests/`
//! or `app/test/` before this target.
//!
//! The claim under test is a **boundary contract**, which is why this is an RI
//! suite rather than a unit test. `memory/tree/tree_runtime/ops.rs` defaults the
//! timestamp host-side, deliberately, and says why:
//!
//!     // Defaulted here rather than driver-side, exactly as before: the reply
//!     // echoes the instant the content was filed under, and a timestamp the
//!     // driver resolved would disagree with the one reported here by however
//!     // long the call took to cross.
//!
//! If that defaulting ever migrates across the bus, the reply starts reporting
//! a different instant than the one the content was actually filed under. The
//! field stays present and stays plausible, so every downstream ordering or
//! dedupe decision keyed on that echo drifts silently. Nothing catches it
//! today.
//!
//! Run with: `cargo test -p openhuman-cli --test in_process_all`

use crate::env_guard::env_lock_async;
use crate::env_guard::EnvVarGuard;
use crate::memory_rpc::{serve, write_config};
use crate::rpc_auth::rpc_token;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::Duration;

use axum::http::header::AUTHORIZATION;
use chrono::{DateTime, SecondsFormat, TimeZone, Utc};
use serde_json::{json, Value};
use tempfile::TempDir;

const NAMESPACE: &str = "tree-summarizer-e2e";

static TEST_HOME: OnceLock<TempDir> = OnceLock::new();

fn test_home() -> &'static Path {
    TEST_HOME
        .get_or_init(|| tempfile::tempdir().expect("tree summarizer tempdir"))
        .path()
}

async fn rpc(base: &str, id: i64, method: &str, params: Value) -> Value {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("client");
    let url = format!("{}/rpc", base.trim_end_matches('/'));
    let resp = client
        .post(&url)
        .header(AUTHORIZATION, format!("Bearer {}", rpc_token()))
        .json(&json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }))
        .send()
        .await
        .unwrap_or_else(|e| panic!("POST {url}: {e}"));
    assert!(
        resp.status().is_success(),
        "HTTP error {} for {method}",
        resp.status()
    );
    resp.json::<Value>()
        .await
        .unwrap_or_else(|e| panic!("json parse for {method}: {e}"))
}

fn ok(v: &Value, ctx: &str) -> Value {
    if let Some(err) = v.get("error") {
        panic!("{ctx}: JSON-RPC error: {err}");
    }
    let outer = v
        .get("result")
        .unwrap_or_else(|| panic!("{ctx}: missing result: {v}"));
    // Outcome wraps its payload under an inner "result" alongside "logs".
    outer
        .get("result")
        .cloned()
        .unwrap_or_else(|| outer.clone())
}

fn setup() -> (EnvVarGuard, EnvVarGuard, EnvVarGuard, EnvVarGuard, PathBuf) {
    let home = test_home();
    let openhuman_home = home.join(".openhuman");
    let home_guard = EnvVarGuard::set_to_path("HOME", home);
    let ws = EnvVarGuard::unset("OPENHUMAN_WORKSPACE");
    let backend = EnvVarGuard::unset("BACKEND_URL");
    let vite = EnvVarGuard::unset("VITE_BACKEND_URL");
    write_config(&openhuman_home);
    (home_guard, ws, backend, vite, openhuman_home)
}

// ── Tests ────────────────────────────────────────────────────────────

/// An explicit `timestamp` is echoed back exactly, not replaced by `now()`.
///
/// The fixture instant is fixed and far in the past, so a handler that resolved
/// its own `now()` could not satisfy this by coincidence.
#[tokio::test]
async fn ingest_echoes_the_caller_supplied_timestamp() {
    let _lock = env_lock_async().await;
    let (_home, _ws, _backend, _vite, _oh) = setup();

    let (rpc_base, _join) = serve().await;
    tokio::time::sleep(Duration::from_millis(100)).await;

    // 2021-03-04T05:06:07Z — fixed, and years away from any plausible `now()`.
    let fixed: DateTime<Utc> = Utc
        .with_ymd_and_hms(2021, 3, 4, 5, 6, 7)
        .single()
        .expect("fixed fixture instant");
    let fixed_rfc3339 = fixed.to_rfc3339_opts(SecondsFormat::Secs, true);

    let before = Utc::now();
    let response = rpc(
        &rpc_base,
        401,
        "openhuman.tree_summarizer_ingest",
        json!({
            "namespace": NAMESPACE,
            "content": "Engineering note filed under a caller-supplied instant.",
            "timestamp": fixed_rfc3339,
        }),
    )
    .await;
    let result = ok(&response, "tree_summarizer_ingest with explicit timestamp");

    // Fixture guard: if the write did not happen there is nothing to assert
    // about, and `buffered` is the handler's own statement that it did.
    assert_eq!(
        result.get("buffered"),
        Some(&json!(true)),
        "ingest did not report buffered=true — got {result}"
    );

    let echoed = result
        .get("timestamp")
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("no `timestamp` in {result}"));
    let echoed: DateTime<Utc> = DateTime::parse_from_rfc3339(echoed)
        .unwrap_or_else(|e| panic!("unparseable timestamp {echoed:?}: {e}"))
        .with_timezone(&Utc);

    assert_eq!(
        echoed, fixed,
        "ingest must echo the instant the caller filed the content under ({fixed}), not one it \
         resolved itself. A driver-side default would disagree with the reply by however long the \
         call took to cross (memory/tree/tree_runtime/ops.rs)."
    );
    // Stated separately so a regression that returns `now()` names itself,
    // rather than only showing two timestamps that happen to differ.
    assert!(
        echoed < before,
        "echoed timestamp {echoed} is not the fixture instant but a value at or after this \
         test started ({before}) — the handler resolved its own clock"
    );

    assert_eq!(
        result.get("namespace"),
        Some(&json!(NAMESPACE)),
        "ingest must echo the namespace it filed under — got {result}"
    );
    assert_eq!(
        result.get("has_metadata"),
        Some(&json!(false)),
        "no metadata was sent, so has_metadata must be false — got {result}"
    );
    let path = result
        .get("path")
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("no `path` in {result}"));
    assert!(
        !path.is_empty(),
        "ingest must report the buffer path the driver wrote"
    );
}

/// With no `timestamp`, the handler resolves one itself and it lands inside the
/// window this test brackets.
///
/// Bracketing both sides is what makes this non-vacuous: a handler returning a
/// constant, a zero instant or an unparseable string fails.
#[tokio::test]
async fn ingest_without_a_timestamp_files_under_the_host_clock() {
    let _lock = env_lock_async().await;
    let (_home, _ws, _backend, _vite, _oh) = setup();

    let (rpc_base, _join) = serve().await;
    tokio::time::sleep(Duration::from_millis(100)).await;

    // One second of slack each side: the assertion is "the host clock", not a
    // claim about RPC latency.
    let before = Utc::now() - chrono::Duration::seconds(1);
    let response = rpc(
        &rpc_base,
        402,
        "openhuman.tree_summarizer_ingest",
        json!({
            "namespace": NAMESPACE,
            "content": "Engineering note filed without an explicit instant.",
            "metadata": { "source": "tree-summarizer-e2e" },
        }),
    )
    .await;
    let after = Utc::now() + chrono::Duration::seconds(1);
    let result = ok(&response, "tree_summarizer_ingest without timestamp");

    assert_eq!(
        result.get("buffered"),
        Some(&json!(true)),
        "ingest did not report buffered=true — got {result}"
    );

    let echoed = result
        .get("timestamp")
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("no `timestamp` in {result}"));
    let echoed: DateTime<Utc> = DateTime::parse_from_rfc3339(echoed)
        .unwrap_or_else(|e| panic!("unparseable timestamp {echoed:?}: {e}"))
        .with_timezone(&Utc);

    assert!(
        echoed >= before && echoed <= after,
        "an omitted timestamp must be defaulted to the host clock; {echoed} is outside the \
         window [{before}, {after}] this call was made in"
    );

    // The metadata this call *did* send must be reflected, so the two tests
    // together pin both branches of `has_metadata`.
    assert_eq!(
        result.get("has_metadata"),
        Some(&json!(true)),
        "metadata was sent, so has_metadata must be true — got {result}"
    );
}
