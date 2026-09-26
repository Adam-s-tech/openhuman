use std::ffi::OsString;
use std::time::Duration;

use tokio_util::sync::CancellationToken;

use crate::server::testing::EnvVarGuard;

#[test]
fn e2e_environment_enables_advertised_tool_groups() {
    let _guard = EnvVarGuard::set_many(vec![("OPENHUMAN_E2E", "1".into())]);
    let builder =
        openhuman_core::core::runtime::CoreBuilder::new(openhuman_core::core::types::HostKind::Cli);
    let _ = super::apply_e2e_tool_groups(builder);
}

async fn wait_until_port_accepts(port: u16) {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
    loop {
        if tokio::net::TcpStream::connect(("127.0.0.1", port))
            .await
            .is_ok()
        {
            return;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "server did not start accepting on port {port}"
        );
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
}

async fn wait_until_port_released(port: u16) {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
    loop {
        if tokio::net::TcpStream::connect(("127.0.0.1", port))
            .await
            .is_err()
        {
            return;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "server did not release port {port}"
        );
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
}

/// Regression test for issue #920 — the embedded server's `axum::serve`
/// accept loop must stop within the cancellation timeout when its
/// `CancellationToken` is fired.
///
/// **Ignored by default.** This test calls `run_server_embedded`,
/// which triggers the full production bootstrap (`bootstrap_core_runtime`
/// → `register_domain_subscribers` → `scheduler_gate::init_global` +
/// `memory::tree::jobs::start` + `composio::start_periodic_sync` +
/// cron scheduler). Those code paths spawn detached `tokio::spawn`
/// background tasks and write to several process-global statics
/// (`STATE: OnceLock`, `SIGNED_OUT: AtomicBool`, `LLM_PERMITS`
/// semaphore, `GLOBAL_REGISTRY` agent.run_turn handler, `STARTED`
/// `std::sync::Once`s, …) — *none of which have teardown semantics*.
/// In a unit-test binary the leaked tasks then race with every other
/// test, multiplying CI wall time by 10–20× (PR #1552 thread). The
/// right shape for this regression is an integration test in a
/// dedicated `tests/` binary where global pollution doesn't affect
/// siblings — tracked as a follow-up.
///
/// To run manually: `cargo test --lib -p openhuman-rpc -- --ignored
/// shutdown_token`.
#[tokio::test]
#[ignore = "calls full server bootstrap; leaks process-global state into sibling tests (#1552). Re-cover via integration test."]
async fn shutdown_token_stops_axum_listener_within_timeout() {
    // Ignored and run on its own, so a plain write needs no restore guard.
    openhuman_core::cron::scheduler_gate::set_signed_out(false);

    let workspace = tempfile::tempdir().expect("workspace tempdir");

    // Pin scheduler-gate policy to Aggressive while this test runs so
    // the bootstrap's `init_global` snapshot can't capture transient
    // CPU pressure and freeze the cached policy at Paused.
    std::fs::write(
        workspace.path().join("config.toml"),
        "[scheduler_gate]\nmode = \"always_on\"\n",
    )
    .expect("seed scheduler_gate=always_on config.toml");
    let _env = EnvVarGuard::set_many(vec![
        (
            "OPENHUMAN_WORKSPACE",
            workspace.path().as_os_str().to_os_string(),
        ),
        ("OPENHUMAN_DISABLE_CHANNEL_LISTENERS", OsString::from("1")),
        (
            "OPENHUMAN_CORE_TOKEN",
            OsString::from("test-token-shutdown"),
        ),
    ]);

    let probe = std::net::TcpListener::bind("127.0.0.1:0").expect("allocate test port");
    let port = probe.local_addr().expect("local addr").port();
    drop(probe);

    let shutdown_token = CancellationToken::new();
    let server_token = shutdown_token.clone();
    let server = tokio::spawn(async move {
        super::run_server_embedded(Some("127.0.0.1"), Some(port), false, server_token).await
    });

    wait_until_port_accepts(port).await;
    shutdown_token.cancel();

    let result = tokio::time::timeout(Duration::from_secs(2), server)
        .await
        .expect("embedded server task should stop within timeout")
        .expect("embedded server task should not panic");
    result.expect("embedded server should shut down cleanly");
    wait_until_port_released(port).await;
}
