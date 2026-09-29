//! Memory-engine selection over JSON-RPC, against a hosted-CortexDB double.
//!
//! Drives `openhuman.memory_engines_list` / `engine_get` / `engine_set` /
//! `engine_migrate` / `engine_migrate_status` through the real HTTP JSON-RPC
//! router, with the TinyHumans backend replaced by an in-test `/memory/*`
//! double (the same dialect as `scripts/mock-api/routes/memory.mjs`, which has
//! its own node tests). The double is in Rust so the suite needs no node
//! process and runs under a plain `cargo test`.
//!
//! ```text
//! RUST_MIN_STACK=67108864 cargo test -p openhuman-cli \
//!   --features "$(bash scripts/ci/product-features.sh)" \
//!   --test memory_engine_e2e
//! ```
//!
//! Serialised on one env lock: `HOME`, `OPENHUMAN_WORKSPACE` and the memory
//! module's captured workspace are process-global.

#[path = "support/memory_module.rs"]
mod memory_module;
#[path = "support/tinyhumans_boot.rs"]
mod tinyhumans_boot;

use std::collections::HashSet;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

use axum::extract::{Query, State};
use axum::http::{header::AUTHORIZATION, HeaderMap, StatusCode};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::{json, Value};
use tempfile::tempdir;

use openhuman_core::core::auth::{init_rpc_token, CORE_TOKEN_ENV_VAR};
use openhuman_core::core::jsonrpc::build_core_http_router;
use openhuman_core::memory::api::provider::MemoryCore;

const TEST_RPC_TOKEN: &str = "memory-engine-e2e-token";
const TEST_API_KEY: &str = "tiny_live_memory_engine_e2e";
const NS: &str = "engine-e2e";

// ── Hosted CortexDB double ──────────────────────────────────────────────────

#[derive(Default)]
struct HostedState {
    events: Mutex<Vec<Value>>,
    idempotency: Mutex<std::collections::BTreeMap<String, (String, String)>>,
    claims: Mutex<HashSet<String>>,
    bearers: Mutex<Vec<String>>,
    next_id: Mutex<u64>,
    /// When non-zero every `/memory/*` call fails with this status.
    force_status: AtomicU16,
}

type Hosted = Arc<HostedState>;

fn err(status: u16, code: &str) -> (StatusCode, Json<Value>) {
    (
        StatusCode::from_u16(status).unwrap(),
        Json(json!({ "success": false, "error": format!("failed: {code}"), "errorCode": code })),
    )
}

fn ok(data: Value) -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({ "success": true, "data": data })))
}

fn gate(state: &Hosted, headers: &HeaderMap) -> Option<(StatusCode, Json<Value>)> {
    let token = headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .unwrap_or_default()
        .to_string();
    state.bearers.lock().unwrap().push(token.clone());
    if token.is_empty() {
        return Some(err(401, "UNAUTHORIZED"));
    }
    match state.force_status.load(Ordering::SeqCst) {
        0 => None,
        402 => Some(err(402, "USER_INSUFFICIENT_CREDITS")),
        other => Some(err(other, "UPSTREAM_ERROR")),
    }
}

async fn experience(
    State(state): State<Hosted>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> (StatusCode, Json<Value>) {
    if let Some(early) = gate(&state, &headers) {
        return early;
    }
    if let Some(claim) = headers.get("idempotency-key").and_then(|v| v.to_str().ok()) {
        if !state.claims.lock().unwrap().insert(claim.to_string()) {
            return err(409, "CONFLICT");
        }
    }
    let key = body["idempotency_key"].as_str().unwrap_or_default().to_string();
    let text = body["content"]["text"].as_str().unwrap_or_default().to_string();
    if let Some((seen, id)) = state.idempotency.lock().unwrap().get(&key) {
        return if seen == &text {
            ok(json!({ "event_id": id, "replayed_from_idempotency": true }))
        } else {
            err(409, "IDEMPOTENCY_CONFLICT")
        };
    }
    let id = {
        let mut next = state.next_id.lock().unwrap();
        *next += 1;
        format!("evt_{}", *next)
    };
    let offset = state.events.lock().unwrap().len() as u64 * 2 + 2;
    state
        .idempotency
        .lock()
        .unwrap()
        .insert(key, (text, id.clone()));
    state.events.lock().unwrap().push(json!({
        "id": id,
        "scope": body["scope"],
        "modality": body["modality"],
        "wal_offset": offset,
        "content": body["content"],
        "context": { "recorded_at": "2026-09-02T00:00:00Z" },
    }));
    ok(json!({ "event_id": id, "status": "captured", "replayed_from_idempotency": false }))
}

async fn events(
    State(state): State<Hosted>,
    headers: HeaderMap,
    Query(params): Query<std::collections::BTreeMap<String, String>>,
) -> (StatusCode, Json<Value>) {
    if let Some(early) = gate(&state, &headers) {
        return early;
    }
    let scope = params.get("scope").cloned().unwrap_or_default();
    let cursor: usize = params.get("cursor").and_then(|v| v.parse().ok()).unwrap_or(0);
    let limit: usize = params.get("limit").and_then(|v| v.parse().ok()).unwrap_or(50);
    let mut stream = Vec::new();
    for event in state.events.lock().unwrap().iter().rev() {
        if event["scope"].as_str() == Some(scope.as_str()) {
            stream.push(event.clone());
            stream.push(event.clone());
        }
    }
    let page: Vec<Value> = stream.iter().skip(cursor).take(limit).cloned().collect();
    let next = cursor + page.len();
    ok(json!({ "items": page, "has_more": next < stream.len(), "next_cursor": next.to_string() }))
}

async fn recall(
    State(state): State<Hosted>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> (StatusCode, Json<Value>) {
    if let Some(early) = gate(&state, &headers) {
        return early;
    }
    let scope = body["scope"].as_str().unwrap_or_default();
    let query = body["query"].as_str().unwrap_or_default().to_lowercase();
    let hits: Vec<Value> = state
        .events
        .lock()
        .unwrap()
        .iter()
        .filter(|e| e["scope"].as_str() == Some(scope))
        .filter(|e| {
            query.is_empty()
                || e["content"]["text"]
                    .as_str()
                    .is_some_and(|t| t.to_lowercase().contains(&query))
        })
        .map(|e| {
            let mut hit = e.clone();
            if let Some(text) = e["content"]["text"].as_str() {
                hit["content"]["text"] = json!(format!("[user] {text}"));
            }
            hit
        })
        .collect();
    ok(json!({ "pack_id": "pack_test", "layers": { "events": hits } }))
}

async fn scopes(
    State(state): State<Hosted>,
    headers: HeaderMap,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> (StatusCode, Json<Value>) {
    if let Some(early) = gate(&state, &headers) {
        return early;
    }
    let limit: usize = params.get("limit").and_then(|v| v.parse().ok()).unwrap_or(50);
    let mut paths: Vec<String> = state
        .events
        .lock()
        .unwrap()
        .iter()
        .filter_map(|e| e["scope"].as_str().map(str::to_string))
        .collect();
    paths.sort();
    paths.dedup();
    paths.truncate(limit);
    ok(json!({ "items": paths.into_iter().map(|p| json!({ "path": p })).collect::<Vec<_>>() }))
}

async fn forget(
    State(state): State<Hosted>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> (StatusCode, Json<Value>) {
    if let Some(early) = gate(&state, &headers) {
        return early;
    }
    let ids: Vec<String> = body["selector"]["memory_ids"]
        .as_array()
        .map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
        .unwrap_or_default();
    let before = state.events.lock().unwrap().len();
    state
        .events
        .lock()
        .unwrap()
        .retain(|e| !ids.contains(&e["id"].as_str().unwrap_or_default().to_string()));
    let deleted = before - state.events.lock().unwrap().len();
    ok(json!({ "deleted": { "events": deleted }, "requested": ids.len(), "matched": deleted }))
}

async fn start_hosted() -> (String, Hosted) {
    let state: Hosted = Arc::new(HostedState::default());
    let app = Router::new()
        .route("/memory/experience", post(experience))
        .route("/memory/events", get(events))
        .route("/memory/recall", post(recall))
        .route("/memory/forget", post(forget))
        .route("/memory/scopes", get(scopes))
        .with_state(state.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await });
    (format!("http://{addr}"), state)
}

// ── Harness ─────────────────────────────────────────────────────────────────

struct EnvVarGuard {
    key: &'static str,
    old: Option<String>,
}

impl EnvVarGuard {
    fn set_to_path(key: &'static str, path: &Path) -> Self {
        let old = std::env::var(key).ok();
        std::env::set_var(key, path.as_os_str());
        Self { key, old }
    }
    fn unset(key: &'static str) -> Self {
        let old = std::env::var(key).ok();
        std::env::remove_var(key);
        Self { key, old }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        match &self.old {
            Some(v) => std::env::set_var(self.key, v),
            None => std::env::remove_var(self.key),
        }
    }
}

static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
static KEYRING_INIT: OnceLock<()> = OnceLock::new();
static AUTH_INIT: OnceLock<()> = OnceLock::new();
static SEAMS_INIT: OnceLock<()> = OnceLock::new();

fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    KEYRING_INIT.get_or_init(|| unsafe {
        std::env::set_var("OPENHUMAN_KEYRING_BACKEND", "file");
    });
    match ENV_LOCK.get_or_init(|| Mutex::new(())).lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

/// The one workspace the process's memory module and every case share.
fn shared_workspace() -> &'static Path {
    static SHARED: OnceLock<PathBuf> = OnceLock::new();
    SHARED.get_or_init(|| {
        let tmp = tempdir().expect("shared workspace tempdir");
        let path = tmp.path().join("workspace");
        std::fs::create_dir_all(&path).expect("shared workspace dir");
        std::mem::forget(tmp);
        path
    })
}

/// The config file that workspace resolves to (legacy sibling layout).
fn shared_config_path() -> PathBuf {
    shared_workspace()
        .parent()
        .expect("workspace parent")
        .join(".openhuman")
        .join("config.toml")
}

fn ensure_seams() {
    SEAMS_INIT.get_or_init(|| {
        std::thread::Builder::new()
            .name("memory-engine-e2e-seams".to_string())
            .stack_size(8 * 1024 * 1024)
            .spawn(|| {
                let config = Arc::new(openhuman_core::config::Config {
                    workspace_dir: shared_workspace().to_path_buf(),
                    ..openhuman_core::config::Config::default()
                });
                #[cfg(feature = "modules")]
                openhuman_core::modules::memory::set_modules_policy(config);
                #[cfg(not(feature = "modules"))]
                let _ = config;
            })
            .expect("spawn seam installer")
            .join()
            .expect("seam installer panicked");
    });
}

fn ensure_rpc_auth() {
    AUTH_INIT.get_or_init(|| {
        tinyhumans_boot::boot();
        unsafe { std::env::set_var(CORE_TOKEN_ENV_VAR, TEST_RPC_TOKEN) };
        let dir = std::env::temp_dir().join("openhuman-memory-engine-e2e-auth");
        init_rpc_token(&dir).expect("init rpc token");
    });
}

fn run_on_big_stack<F, Fut>(name: &str, factory: F)
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = ()> + 'static,
{
    std::thread::Builder::new()
        .name(name.to_string())
        .stack_size(openhuman_core::core::runtime::AGENT_WORKER_STACK_BYTES)
        .spawn(move || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .thread_stack_size(openhuman_core::core::runtime::AGENT_WORKER_STACK_BYTES)
                .enable_all()
                .build()
                .expect("runtime");
            rt.block_on(factory());
        })
        .expect("spawn")
        .join()
        .expect("test body panicked");
}

async fn rpc(base: &str, method: &str, params: Value) -> Value {
    let resp = reqwest::Client::builder()
        .timeout(Duration::from_secs(120))
        .build()
        .unwrap()
        .post(format!("{base}/rpc"))
        .header(AUTHORIZATION, format!("Bearer {TEST_RPC_TOKEN}"))
        .json(&json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params }))
        .send()
        .await
        .unwrap_or_else(|e| panic!("POST {method}: {e}"));
    assert!(resp.status().is_success(), "HTTP {} for {method}", resp.status());
    resp.json().await.expect("json body")
}

fn result_of<'a>(v: &'a Value, ctx: &str) -> &'a Value {
    if let Some(e) = v.get("error") {
        panic!("{ctx}: JSON-RPC error: {e}");
    }
    let r = v.get("result").unwrap_or_else(|| panic!("{ctx}: no result: {v}"));
    if r.get("logs").is_some() {
        r.get("result").unwrap_or(r)
    } else {
        r
    }
}

fn error_message(v: &Value, ctx: &str) -> String {
    v.pointer("/error/message")
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("{ctx}: expected a JSON-RPC error, got {v}"))
        .to_string()
}

/// Everything a case needs: the hosted double, the RPC base URL and the env.
struct Fixture {
    _lock: std::sync::MutexGuard<'static, ()>,
    _guards: Vec<EnvVarGuard>,
    _tmp: tempfile::TempDir,
    hosted: Hosted,
    origin: String,
    base: String,
    join: tokio::task::JoinHandle<Result<(), std::io::Error>>,
}

impl Fixture {
    async fn new() -> Self {
        let lock = env_lock();
        let tmp = tempdir().expect("home tempdir");
        let guards = vec![
            EnvVarGuard::set_to_path("HOME", tmp.path()),
            EnvVarGuard::set_to_path("OPENHUMAN_WORKSPACE", shared_workspace()),
            EnvVarGuard::unset("OPENHUMAN_MEMORY_DRIVER"),
            EnvVarGuard::unset("BACKEND_URL"),
            EnvVarGuard::unset("VITE_BACKEND_URL"),
            EnvVarGuard::unset("OPENHUMAN_BACKEND_API_KEY"),
        ];
        ensure_rpc_auth();
        ensure_seams();
        let (origin, hosted) = start_hosted().await;

        // A fresh config every case: the previous case's committed engine must
        // not leak into this one.
        let config_path = shared_config_path();
        std::fs::create_dir_all(config_path.parent().unwrap()).unwrap();
        std::fs::write(
            &config_path,
            format!(
                "api_url = \"{origin}\"\ndefault_model = \"e2e-mock-model\"\n\n[secrets]\nencrypt = false\n"
            ),
        )
        .unwrap();

        memory_module::settle().await;
        let config = openhuman_core::config::load_config_with_timeout()
            .await
            .expect("load config");
        openhuman_core::security::credentials::api_key::store_api_key(&config, TEST_API_KEY)
            .expect("store the backend api key");

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let app = build_core_http_router(false);
        let join = tokio::spawn(async move { axum::serve(listener, app).await });
        Self {
            _lock: lock,
            _guards: guards,
            _tmp: tmp,
            hosted,
            origin,
            base: format!("http://{addr}"),
            join,
        }
    }

    async fn call(&self, method: &str, params: Value) -> Value {
        rpc(&self.base, method, params).await
    }

    async fn state(&self) -> Value {
        let v = self.call("openhuman.memory_engine_get", json!({})).await;
        result_of(&v, "engine_get").clone()
    }

    /// Poll a migration job to a terminal state.
    async fn wait_job(&self, job_id: &str) -> Value {
        for _ in 0..200 {
            let v = self
                .call("openhuman.memory_engine_migrate_status", json!({ "job_id": job_id }))
                .await;
            let status = result_of(&v, "engine_migrate_status").clone();
            if status["state"] != "running" {
                return status;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        panic!("migration job {job_id} did not finish");
    }

    async fn put_doc(&self, key: &str, content: &str) {
        let v = self
            .call(
                "openhuman.memory_doc_put",
                json!({
                    "namespace": NS, "key": key, "title": key, "content": content,
                    "source_type": "doc", "priority": "medium", "tags": [],
                    "metadata": null, "category": "core"
                }),
            )
            .await;
        result_of(&v, "doc_put");
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        self.join.abort();
    }
}

// ── Cases ───────────────────────────────────────────────────────────────────

#[test]
fn engines_list_and_get_report_the_module_by_default() {
    run_on_big_stack("engines-list", || async {
        let fx = Fixture::new().await;
        let v = fx.call("openhuman.memory_engines_list", json!({})).await;
        let list = result_of(&v, "engines_list");
        let engines = list["engines"].as_array().expect("engines array");
        let ids: Vec<&str> = engines.iter().filter_map(|e| e["id"].as_str()).collect();

        assert_eq!(ids[0], "tinymemory", "module first: {ids:?}");
        assert_eq!(engines[0]["label"], "TinyCortex (local)");
        for expected in ["tinyhumans", "supermemory", "mem0", "cognee", "cortex", "agentmemory"] {
            assert!(ids.contains(&expected), "{expected} missing from {ids:?}");
        }
        assert!(!ids.contains(&"null"), "null must not be offered: {ids:?}");
        assert!(!ids.contains(&"tinycortex"), "in-memory tinycortex must not be offered: {ids:?}");
        assert_eq!(list["active"], "tinymemory");
        let hosted = engines.iter().find(|e| e["id"] == "tinyhumans").unwrap();
        assert_eq!(hosted["hosted"], true);
        assert!(hosted["capabilities"].as_array().unwrap().iter().any(|c| c == "answer"));

        let state = fx.state().await;
        assert_eq!(state["driver"], "tinymemory");
        assert_eq!(state["class"], "module");
        assert_eq!(state["has_credential"], false);
        assert!(state["fell_back_from"].is_null());
        assert!(state["last_error"].is_null());
    });
}

#[test]
fn engine_set_binds_the_hosted_engine_without_a_restart_and_switches_back() {
    run_on_big_stack("engine-set", || async {
        let fx = Fixture::new().await;

        let v = fx
            .call("openhuman.memory_engine_set", json!({ "driver": "tinyhumans" }))
            .await;
        let state = result_of(&v, "engine_set tinyhumans");
        assert_eq!(state["driver"], "tinyhumans", "{state}");
        assert_eq!(state["class"], "external");
        assert_eq!(state["has_credential"], true, "the live api key is the credential");
        assert_eq!(state["endpoint"], fx.origin.as_str());
        assert!(state["fell_back_from"].is_null(), "{state}");
        assert!(!state.to_string().contains(TEST_API_KEY), "a secret leaked: {state}");

        // The switch persisted: a fresh `get` (new config load) sees it.
        assert_eq!(fx.state().await["driver"], "tinyhumans");
        let status = fx.call("openhuman.memory_provider_status", json!({})).await;
        assert_eq!(result_of(&status, "provider_status")["driver"], "tinyhumans");

        // A write through the bound provider reaches the double under the
        // session bearer, and reads back through the normal recall RPC.
        let config = openhuman_core::config::load_config_with_timeout().await.unwrap();
        let binding = openhuman_core::memory::binding::for_config(&config).unwrap();
        assert_eq!(binding.driver_id(), "tinyhumans");
        binding
            .provider()
            .store(
                NS,
                "engine-note",
                "the hosted engine stores this",
                openhuman_core::memory::api::types::MemoryCategory::Core,
                None,
                openhuman_core::memory::api::types::MemoryTaint::Internal,
            )
            .await
            .expect("store through the hosted engine");
        assert!(
            fx.hosted.events.lock().unwrap().iter().any(|e| e["content"]["text"]
                .as_str()
                .is_some_and(|t| t.contains("the hosted engine stores this"))),
            "the write must land in the hosted double"
        );
        assert!(
            fx.hosted.bearers.lock().unwrap().iter().all(|b| b == TEST_API_KEY),
            "every hosted call carries the live credential"
        );
        // Reads go through the mandatory families: recall by query, and the
        // namespace listing the RPC layer serves.
        let recalled = binding
            .provider()
            .recall(
                "hosted engine",
                5,
                &openhuman_core::memory::api::recall::OwnedRecallOpts {
                    namespace: Some(NS.to_string()),
                    ..Default::default()
                },
                None,
            )
            .await
            .expect("recall through the hosted engine");
        assert!(
            recalled.iter().any(|e| e.content.contains("the hosted engine stores this")),
            "recall must be served by the hosted engine: {recalled:?}"
        );
        let namespaces = fx.call("openhuman.memory_list_namespaces", json!({})).await;
        assert!(
            namespaces.to_string().contains(NS),
            "the namespace listing must come from the hosted engine: {namespaces}"
        );

        // Families the hosted engine does not advertise answer with a clean
        // error envelope, never a panic or a hang. Probe a few and print what
        // each answers (visible with --nocapture) for the degradation report.
        for (method, params) in [
            ("openhuman.memory_recall_memories", json!({ "namespace": NS, "limit": 5 })),
            ("openhuman.memory_recall_context", json!({ "namespace": NS, "limit": 5 })),
            ("openhuman.memory_query_namespace", json!({ "namespace": NS, "query": "hosted" })),
            ("openhuman.memory_doc_list", json!({ "namespace": NS })),
            ("openhuman.memory_graph_query", json!({ "namespace": NS })),
        ] {
            let v = fx.call(method, params).await;
            let outcome = if v.get("error").is_some() {
                format!("jsonrpc error: {}", v["error"]["message"])
            } else if let Some(e) = v.pointer("/result/error/message") {
                format!("clean error: {e}")
            } else {
                "served".to_string()
            };
            eprintln!("[degrade-probe] hosted engine {method}: {outcome}");
        }

        // An unsupported family degrades to a clean error, not a panic.
        let doc = fx
            .call(
                "openhuman.memory_doc_put",
                json!({
                    "namespace": NS, "key": "k", "title": "t", "content": "c",
                    "source_type": "doc", "priority": "medium", "tags": [],
                    "metadata": null, "category": "core"
                }),
            )
            .await;
        // Either the RPC is unregistered for this capability set or the handler
        // reports the missing family; both are clean errors.
        assert!(doc.get("error").is_some(), "doc_put on the hosted engine: {doc}");

        // And back to the module, live.
        let v = fx
            .call("openhuman.memory_engine_set", json!({ "driver": "tinymemory" }))
            .await;
        let state = result_of(&v, "engine_set tinymemory");
        assert_eq!(state["driver"], "tinymemory");
        assert_eq!(state["class"], "module");
    });
}

#[test]
fn engine_set_validates_and_never_stores_or_returns_a_key_in_config() {
    run_on_big_stack("engine-set-validate", || async {
        let fx = Fixture::new().await;

        for bad in ["not-an-engine", "null", ""] {
            let v = fx
                .call("openhuman.memory_engine_set", json!({ "driver": bad }))
                .await;
            let message = error_message(&v, &format!("engine_set {bad:?}"));
            assert!(!message.is_empty());
        }
        let v = fx
            .call(
                "openhuman.memory_engine_set",
                json!({ "driver": "supermemory", "endpoint": "ftp://nope" }),
            )
            .await;
        assert!(error_message(&v, "bad endpoint").contains("http(s)"));
        let v = fx
            .call(
                "openhuman.memory_engine_set",
                json!({ "driver": "mem0", "deployment": "moon" }),
            )
            .await;
        assert!(error_message(&v, "bad deployment").contains("deployment"));
        assert_eq!(fx.state().await["driver"], "tinymemory", "a rejected set changes nothing");

        let secret = "sm_secret_key_that_must_never_leak";
        let v = fx
            .call(
                "openhuman.memory_engine_set",
                json!({
                    "driver": "supermemory",
                    "endpoint": "https://api.supermemory.ai",
                    "api_key": secret
                }),
            )
            .await;
        let state = result_of(&v, "engine_set supermemory");
        assert_eq!(state["driver"], "supermemory", "{state}");
        assert_eq!(state["class"], "external");
        assert_eq!(state["has_credential"], true);
        assert!(!state.to_string().contains(secret), "key echoed in response: {state}");
        let toml = std::fs::read_to_string(shared_config_path()).unwrap();
        assert!(!toml.contains(secret), "key written to config.toml");
        assert!(toml.contains("keychain:memory-supermemory"), "config should carry the ref: {toml}");
        assert!(toml.contains("trust_state = \"trusted\""), "{toml}");

        fx.call("openhuman.memory_engine_set", json!({ "driver": "tinymemory" })).await;
    });
}

#[test]
fn engine_migrate_copies_the_module_into_the_hosted_engine_then_switches() {
    run_on_big_stack("engine-migrate", || async {
        let fx = Fixture::new().await;
        fx.put_doc("migrate-canary", "canary fact carried across engines").await;

        let v = fx
            .call(
                "openhuman.memory_engine_migrate",
                json!({ "to": { "driver": "tinyhumans" } }),
            )
            .await;
        let job_id = result_of(&v, "engine_migrate")["job_id"]
            .as_str()
            .expect("job_id")
            .to_string();
        let status = fx.wait_job(&job_id).await;
        assert_eq!(status["state"], "done", "{status}");
        assert!(status["error"].is_null());
        assert!(status["copied"].as_u64().unwrap_or(0) >= 1, "{status}");

        assert!(
            fx.hosted.events.lock().unwrap().iter().any(|e| e["content"]["text"]
                .as_str()
                .is_some_and(|t| t.contains("canary fact carried across engines"))),
            "the migrated record must exist in the hosted engine"
        );
        let state = fx.state().await;
        assert_eq!(state["driver"], "tinyhumans", "the switch commits after the copy: {state}");

        let unknown = fx
            .call("openhuman.memory_engine_migrate_status", json!({ "job_id": "nope" }))
            .await;
        assert!(error_message(&unknown, "unknown job").contains("unknown"));

        // Migrating to the engine already in use is refused.
        let again = fx
            .call(
                "openhuman.memory_engine_migrate",
                json!({ "to": { "driver": "tinyhumans" } }),
            )
            .await;
        assert!(error_message(&again, "same engine").contains("already"));

        fx.call("openhuman.memory_engine_set", json!({ "driver": "tinymemory" })).await;
    });
}

#[test]
fn insufficient_credits_fails_the_migration_and_keeps_the_active_engine() {
    run_on_big_stack("engine-migrate-402", || async {
        let fx = Fixture::new().await;
        fx.put_doc("credits-canary", "record that cannot be copied").await;
        fx.hosted.force_status.store(402, Ordering::SeqCst);

        let v = fx
            .call(
                "openhuman.memory_engine_migrate",
                json!({ "to": { "driver": "tinyhumans" } }),
            )
            .await;
        let job_id = result_of(&v, "engine_migrate")["job_id"].as_str().unwrap().to_string();
        let status = fx.wait_job(&job_id).await;
        assert_eq!(status["state"], "failed", "{status}");
        let message = status["error"].as_str().expect("failure reason");
        assert!(
            message.starts_with("INSUFFICIENT_CREDITS:"),
            "402 must map to INSUFFICIENT_CREDITS: {message}"
        );

        let state = fx.state().await;
        assert_eq!(state["driver"], "tinymemory", "a failed migration must not switch: {state}");
        assert_eq!(state["class"], "module");
    });
}
