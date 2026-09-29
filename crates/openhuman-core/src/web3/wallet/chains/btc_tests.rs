use super::*;
use crate::web3::wallet::execution::{
    insert_quote_for_test, now_ms, reset_quote_store_for_tests, PreparedKind, PreparedStatus,
    PreparedTransaction,
};
use crate::web3::wallet::test_support::{sample_btc_address, setup_wallet_in, TEST_LOCK};
use axum::{
    routing::{get, post},
    Router,
};
use serde_json::json;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::net::TcpListener;

#[test]
fn select_utxos_largest_first_returns_change() {
    let utxos = vec![
        EsploraUtxo {
            txid: "a".into(),
            vout: 0,
            value: 5000,
        },
        EsploraUtxo {
            txid: "b".into(),
            vout: 0,
            value: 10_000,
        },
        EsploraUtxo {
            txid: "c".into(),
            vout: 0,
            value: 1_000,
        },
    ];
    let (chosen, change) = select_utxos(&utxos, 6_000, 2_000).unwrap();
    assert_eq!(chosen.len(), 1);
    assert_eq!(chosen[0].txid, "b");
    assert_eq!(change, 2_000);
}

#[test]
fn select_utxos_combines_multiple_when_needed() {
    let utxos = vec![
        EsploraUtxo {
            txid: "a".into(),
            vout: 0,
            value: 5000,
        },
        EsploraUtxo {
            txid: "b".into(),
            vout: 0,
            value: 5000,
        },
        EsploraUtxo {
            txid: "c".into(),
            vout: 0,
            value: 5000,
        },
    ];
    let (chosen, change) = select_utxos(&utxos, 11_000, 1_000).unwrap();
    assert_eq!(chosen.len(), 3);
    assert_eq!(change, 3_000);
}

#[test]
fn select_utxos_errors_when_insufficient() {
    let utxos = vec![EsploraUtxo {
        txid: "a".into(),
        vout: 0,
        value: 1_000,
    }];
    let err = select_utxos(&utxos, 5_000, 1_000).unwrap_err();
    assert!(err.contains("insufficient"), "got: {err}");
}

// Drives the real wallet module, so it must be the only such test in its
// process: tinybus never unloads a module, and the module bus belongs to
// whichever tokio runtime created it — a second `#[tokio::test]` finds a
// broker whose tasks died with the first and the call fails with
// "connection closed". Verified passing in isolation:
//
//   cargo test -p openhuman --lib --features "$(bash scripts/ci/product-features.sh)" \
//     execute_btc_quote_builds_psbt_signs_and_broadcasts -- --ignored --test-threads=1
//
// Same constraint tinydocs documents for its module-backed tool tests.
#[ignore = "drives the loaded wallet module; must run alone in its process"]
#[tokio::test]
async fn execute_btc_quote_builds_psbt_signs_and_broadcasts() {
    let _guard = TEST_LOCK.lock();
    reset_quote_store_for_tests();
    let temp = TempDir::new().unwrap();
    let _workspace_guard = setup_wallet_in(&temp).await.unwrap();

    // Mock state: collect raw tx hex posted to /tx.
    let raw_txs: Arc<parking_lot::Mutex<Vec<String>>> =
        Arc::new(parking_lot::Mutex::new(Vec::new()));
    let raw_txs_clone = raw_txs.clone();
    let from_addr = sample_btc_address().to_string();
    // Real-shaped UTXO; value high enough to cover amount + fee.
    let utxo_txid = "1111111111111111111111111111111111111111111111111111111111111111";
    let utxo_json = json!([
        { "txid": utxo_txid, "vout": 0, "value": 100_000u64 }
    ]);
    let utxo_clone = utxo_json.clone();
    let app = Router::new()
        .route(
            "/address/{addr}/utxo",
            get(move || {
                let body = utxo_clone.clone();
                async move { axum::Json(body) }
            }),
        )
        .route(
            "/tx",
            post(move |body: String| {
                let raw_txs = raw_txs_clone.clone();
                async move {
                    raw_txs.lock().push(body);
                    // Return a known fake txid the test can assert on.
                    "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef".to_string()
                }
            }),
        );
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    std::env::set_var("OPENHUMAN_WALLET_RPC_BTC", format!("http://{addr}"));

    let now = now_ms();
    let quote = PreparedTransaction {
        quote_id: "q_btc_native_1".to_string(),
        kind: PreparedKind::NativeTransfer,
        chain: WalletChain::Btc,
        evm_network: None,
        from_address: from_addr.clone(),
        to_address: "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4".to_string(),
        asset_symbol: "BTC".to_string(),
        amount_raw: "50000".to_string(),
        amount_formatted: "0.00050000".to_string(),
        receive_symbol: None,
        min_receive_raw: None,
        calldata: None,
        token_address: None,
        estimated_fee_raw: "5000".to_string(),
        status: PreparedStatus::AwaitingConfirmation,
        created_at_ms: now,
        expires_at_ms: now + 60_000,
        notes: vec![],
        owner: None,
    };
    insert_quote_for_test(quote.clone());

    let result = execute_btc_quote(quote).await.expect("btc broadcast ok");
    assert_eq!(result.status, PreparedStatus::Broadcasted);
    assert_eq!(
        result.transaction_hash,
        "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef"
    );
    let raw = raw_txs.lock().clone();
    assert_eq!(raw.len(), 1, "exactly one broadcast call");
    let tx_hex = &raw[0];
    assert!(!tx_hex.is_empty(), "tx hex must be non-empty");
    // Witness-segwit transactions include the BIP141 marker+flag (0x0001).
    assert!(
        tx_hex.contains("0001"),
        "expected segwit marker, got: {tx_hex}"
    );
}

#[tokio::test]
async fn execute_btc_quote_rejects_insufficient_utxos() {
    let _guard = TEST_LOCK.lock();
    reset_quote_store_for_tests();
    let temp = TempDir::new().unwrap();
    let _workspace_guard = setup_wallet_in(&temp).await.unwrap();

    // Empty UTXO set — must error.
    let app = Router::new().route(
        "/address/{addr}/utxo",
        get(|| async { axum::Json(json!([])) }),
    );
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    std::env::set_var("OPENHUMAN_WALLET_RPC_BTC", format!("http://{addr}"));

    let now = now_ms();
    let quote = PreparedTransaction {
        quote_id: "q_btc_native_empty".to_string(),
        kind: PreparedKind::NativeTransfer,
        chain: WalletChain::Btc,
        evm_network: None,
        from_address: sample_btc_address().to_string(),
        to_address: "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4".to_string(),
        asset_symbol: "BTC".to_string(),
        amount_raw: "50000".to_string(),
        amount_formatted: "0.00050000".to_string(),
        receive_symbol: None,
        min_receive_raw: None,
        calldata: None,
        token_address: None,
        estimated_fee_raw: "5000".to_string(),
        status: PreparedStatus::AwaitingConfirmation,
        created_at_ms: now,
        expires_at_ms: now + 60_000,
        notes: vec![],
        owner: None,
    };
    let err = execute_btc_quote(quote).await.unwrap_err();
    assert!(err.contains("no spendable UTXOs"), "got: {err}");
}

#[test]
fn derive_btc_key_produces_known_p2wpkh_from_test_mnemonic() {
    // BIP84 m/84'/0'/0'/0/0 from "abandon x11 about" → bc1qcr8...
    // The compressed pubkey should serialize to 33 bytes.
    let mnemonic =
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    let (secret, pubkey) = derive_btc_private_key(mnemonic, "m/84'/0'/0'/0/0").unwrap();
    assert_eq!(secret.len(), 32);
    // Compressed SEC1. The uncompressed form would be 65 bytes and would
    // hash to a different — spendable by nobody — address.
    assert_eq!(pubkey.len(), 33);
    assert!(matches!(pubkey[0], 0x02 | 0x03));

    // The known-good vector for this mnemonic and path, unchanged by the
    // move off the `bitcoin` crate. Derived through the root `tinywallet`
    // crate, which is the same code that produces the address in
    // `execute_btc_quote` — in production it runs inside the wallet module
    // rather than here, but it is the same crate and the same walk.
    let derived =
        tinywallet::key::derive(tinywallet::Chain::Btc, mnemonic, "m/84'/0'/0'/0/0").unwrap();
    assert_eq!(
        derived.address(),
        "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu"
    );
    assert_eq!(derived.secret_bytes(), secret.as_slice());
}

#[tokio::test]
async fn tx_status_confirmed_with_tip_confirmations() {
    let _guard = TEST_LOCK.lock();
    let _env_guard = crate::config::TEST_ENV_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let app = Router::new()
        .route(
            "/tx/{txid}/status",
            get(|| async { axum::Json(json!({"confirmed": true, "block_height": 800_000u64})) }),
        )
        .route("/blocks/tip/height", get(|| async { "800002" }));
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    std::env::set_var("OPENHUMAN_WALLET_RPC_BTC", format!("http://{addr}"));
    let info = tx_status("deadbeef").await.unwrap();
    assert_eq!(
        info.state,
        crate::web3::wallet::execution::TxState::Confirmed
    );
    assert_eq!(info.block_number, Some(800_000));
    assert_eq!(info.confirmations, Some(3));
}

#[tokio::test]
async fn lookup_tx_not_found_on_404() {
    let _guard = TEST_LOCK.lock();
    let _env_guard = crate::config::TEST_ENV_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let app = Router::new().route(
        "/tx/{txid}",
        get(|| async { (axum::http::StatusCode::NOT_FOUND, "Transaction not found") }),
    );
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    std::env::set_var("OPENHUMAN_WALLET_RPC_BTC", format!("http://{addr}"));
    let info = lookup_tx("deadbeef").await.unwrap();
    assert!(!info.found);
}
