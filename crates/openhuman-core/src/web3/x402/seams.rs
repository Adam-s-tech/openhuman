//! The host side of the `tinywallet-x402` seams.
//!
//! The crate never sees a mnemonic, a config or a proxy setting. It is handed:
//!
//! - a [`PaymentSigner`]: [`WalletPaymentSigner`] resolves the wallet's
//!   encrypted secret from the keyring state, decrypts it with the configured
//!   key, and asks the loaded `tinywallet` module to derive the account and sign.
//!   The decrypted phrase lives only for the length of one confidential call;
//!   what crosses back to the crate is an address and finished signatures.
//! - a `Transport`: [`OpenHumanTransport`](crate::web3::wallet::transport::OpenHumanTransport),
//!   the failover-aware RPC adapter the wallet already uses, for the Solana
//!   blockhash.
//! - a [`ProxyPolicy`]: [`RuntimeProxyPolicy`] applies the runtime proxy
//!   configuration to the tool's HTTP client.
//!
//! Every `Err(String)` these return is the text a user sees, prefixed exactly as
//! the payment flow did before it moved into the crate (`wallet secret: …`,
//! `load config: …`, `decrypt mnemonic: …`, `derive account: …`).

use std::sync::Arc;

use async_trait::async_trait;
use log::debug;
use tinywallet_x402::crypto::{CryptoPayments, PaymentAccount, PaymentSigner, SignScheme};
use tinywallet_x402::protocol::ProxyPolicy;
use tinywallet_x402::tools::X402RequestTool;
use tinywallet_x402::wire::PaymentChain;

use crate::web3::wallet::transport::OpenHumanTransport;
use crate::web3::wallet::WalletChain;

const LOG_PREFIX: &str = "[x402::seams]";

/// The wallet as x402 signs with it: keyring secret, decrypt, wallet module.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct WalletPaymentSigner;

/// The runtime proxy configuration, applied to x402's outbound HTTP.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct RuntimeProxyPolicy;

/// The wallet chain a payment chain is signed as.
fn wallet_chain(chain: PaymentChain) -> WalletChain {
    match chain {
        PaymentChain::Solana => WalletChain::Solana,
        PaymentChain::Evm => WalletChain::Evm,
    }
}

/// The bus chain the wallet module derives for.
fn bus_chain(chain: PaymentChain) -> tinywallet_bus::Chain {
    match chain {
        PaymentChain::Solana => tinywallet_bus::Chain::Solana,
        PaymentChain::Evm => tinywallet_bus::Chain::Evm,
    }
}

/// Resolve the phrase to sign with: the config, and the wallet's decrypted
/// mnemonic packaged for a confidential call.
///
/// Derivation and signing both happen inside the loaded wallet module, so this
/// process never holds a private key. The phrase is handed over on a
/// confidential call, and only to a module that has proved it is an artifact this
/// build pinned.
async fn signing_secret(
    chain: PaymentChain,
) -> Result<(crate::config::Config, tinywallet_bus::wire::SecretMaterial), String> {
    let secret = crate::web3::wallet::secret_material(wallet_chain(chain))
        .await
        .map_err(|e| format!("wallet secret: {e}"))?;

    let config = crate::config::rpc::load_config_with_timeout()
        .await
        .map_err(|e| format!("load config: {e}"))?;

    let mnemonic =
        crate::security::encryption::rpc::decrypt_secret(&config, &secret.encrypted_mnemonic)
            .await
            .map_err(|e| format!("decrypt mnemonic: {e}"))?
            .value;

    Ok((
        config,
        tinywallet_bus::wire::SecretMaterial {
            mnemonic,
            derivation_path: secret.derivation_path.clone(),
            chain: bus_chain(chain),
        },
    ))
}

#[async_trait]
impl PaymentSigner for WalletPaymentSigner {
    async fn account(&self, chain: PaymentChain) -> Result<PaymentAccount, String> {
        debug!("{LOG_PREFIX} account chain={chain:?}");
        let (config, secret) = signing_secret(chain).await?;
        let account = crate::modules::wallet::derive_account(&config, &secret)
            .await
            .map_err(|e| match chain {
                PaymentChain::Solana => format!("derive account: {e}"),
                PaymentChain::Evm => format!("derive EVM signer: {e}"),
            })?;
        // The module answers with the address only; the crate decodes a Solana
        // public key from it.
        Ok(PaymentAccount {
            address: account.address,
            pubkey: None,
        })
    }

    async fn sign(
        &self,
        chain: PaymentChain,
        message: &[u8],
        scheme: SignScheme,
    ) -> Result<Vec<u8>, String> {
        debug!(
            "{LOG_PREFIX} sign chain={chain:?} scheme={scheme:?} bytes={}",
            message.len()
        );
        let (config, secret) = signing_secret(chain).await?;
        let wire_scheme = match scheme {
            SignScheme::Ed25519 => tinywallet_bus::wire::Scheme::Ed25519,
            SignScheme::Secp256k1Digest => tinywallet_bus::wire::Scheme::Secp256k1Prehash,
        };
        // Signed in the module; the private key is never assembled here.
        let signature =
            crate::modules::wallet::sign_message(&config, &secret, message, wire_scheme)
                .await
                .map_err(|e| e.to_string())?;
        signature_bytes(signature)
    }
}

/// The raw bytes the crate expects from the module's signature: 64 for ed25519,
/// `r ‖ s ‖ recovery_id` (65) for secp256k1.
fn signature_bytes(signature: tinywallet_bus::wire::Signature) -> Result<Vec<u8>, String> {
    match signature {
        tinywallet_bus::wire::Signature::Ed25519 { signature_hex } => {
            hex::decode(&signature_hex).map_err(|e| format!("invalid signature hex: {e}"))
        }
        tinywallet_bus::wire::Signature::Secp256k1 {
            rs_hex,
            recovery_id,
        } => {
            let mut bytes =
                hex::decode(&rs_hex).map_err(|e| format!("invalid signature hex: {e}"))?;
            if bytes.len() != 64 {
                return Err("the wallet module returned a malformed signature".to_string());
            }
            bytes.push(recovery_id);
            Ok(bytes)
        }
        // `Signature` is non-exhaustive; a scheme this build has never heard of
        // cannot be paid with.
        _ => Err("the wallet module returned an unsupported signature".to_string()),
    }
}

impl ProxyPolicy for RuntimeProxyPolicy {
    fn apply(&self, builder: reqwest::ClientBuilder, service: &str) -> reqwest::ClientBuilder {
        crate::config::apply_runtime_proxy_to_builder(builder, service)
    }
}

/// The crypto rail's payment builder, over OpenHuman's wallet and transport.
pub(crate) fn payments() -> CryptoPayments {
    CryptoPayments::new(
        Arc::new(WalletPaymentSigner),
        Arc::new(OpenHumanTransport::new()),
    )
}

/// The `x402_request` tool, over the same seams.
pub(crate) fn request_tool() -> X402RequestTool {
    X402RequestTool::new(
        Arc::new(WalletPaymentSigner),
        Arc::new(OpenHumanTransport::new()),
        Arc::new(RuntimeProxyPolicy),
    )
}

#[cfg(test)]
#[path = "seams_tests.rs"]
mod tests;
