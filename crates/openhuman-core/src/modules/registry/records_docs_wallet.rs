//! Registry records for the `tinydocs` and `tinywallet` modules.

use crate::modules::types::{LoadPolicy, ModuleRecord, PlatformAsset};

/// The `tinydocs` module: `.docx` / `.pptx` synthesis and `.pdf` extraction.
///
/// Lazy, because a user who never asks for a document should not pay a download,
/// a `dlopen`, and the resident cost of a library that is never unloaded.
pub(crate) const TINYDOCS: ModuleRecord = ModuleRecord {
    id: "tinydocs",
    description: "Document synthesis (.docx, .pptx) and PDF text extraction",
    bus_name: "ai.tinyhumans.tinydocs.Documents",
    object_path: "/ai/tinyhumans/tinydocs/Documents",
    version: "0.1.20",
    release_url: "https://github.com/tinyhumansai/tinydocs/releases/tag/v0.1.20",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinydocs-module-0.1.20-ubuntu-24.04-x86_64.tar.gz",
            sha256: "685885800efa9d378032dd868243adcd08110fa1040b3855393d68bb0d7a8474",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinydocs-module-0.1.20-ubuntu-24.04-arm64.tar.gz",
            sha256: "af589669ea697585f7e815c59188c7c382173ffad9a37c4f7adf8641d3e13760",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinydocs-module-0.1.20-ubuntu-22.04-x86_64.tar.gz",
            sha256: "86d358b2c497e92082e46a012bef6d09e9420a37347023bcfbe94bb25b6c4a45",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinydocs-module-0.1.20-ubuntu-22.04-arm64.tar.gz",
            sha256: "1afe105f4c11992c0a2c2c1be314a7409964a319301a9ba9074bf5b14ae5bc72",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinydocs-module-0.1.20-macos-26-arm64.tar.gz",
            sha256: "c3984b10ccdfc2bd08a898da85c9a97a22dbaa70a7ead07ed69bea9322d9546d",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinydocs-module-0.1.20-macos-26-x86_64.tar.gz",
            sha256: "e5540064198d081d1b318cc77f4879dd082ac9aabcf5cf68513b782e2a9c6b4a",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinydocs-module-0.1.20-macos-15-arm64.tar.gz",
            sha256: "dd919b6cfd5dc791807fef558cae2f10faa26f463d6f151b8a5c7578b6375a7b",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinydocs-module-0.1.20-macos-15-x86_64.tar.gz",
            sha256: "eeccb271aac7549c1b18ee8cfec8a3f78cdb5bd4363814d45f42d6a333db10f5",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinydocs-module-0.1.20-windows-2025-x86_64.zip",
            sha256: "aaab0cb81ad847faa400091de586e007b3cd06b854c4d4059310f489383b5be1",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinydocs-module-0.1.20-windows-2022-x86_64.zip",
            sha256: "3b40bac38727ecddaee661914e23560ea04513ce4eec8d2aa95b595de35c51f7",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinydocs-module-0.1.20-windows-11-arm64.zip",
            sha256: "2829d11cdfbc790f9b45f6f5c7fd817a613080c3dca21aee27160afef455ddfb",
        },
    ],
    load: LoadPolicy::Lazy,
};

/// The `tinywallet` module: transaction building and assembly for four chains.
///
/// Lazy for the same reason as [`TINYDOCS`], and more so: most sessions never
/// touch a wallet, and this artifact carries `bitcoin` and a native `secp256k1`
/// build that would otherwise be resident for all of them.
///
/// **This host sends it the recovery phrase, over confidential calls, and never
/// derives or signs itself.** All four chains — Bitcoin, EVM, Solana and Tron —
/// derive and sign inside the module. This binary does not link the root
/// `tinywallet` crate at all — it takes `tinywallet-bus`, the wire contract,
/// which carries no `key` gate — nor does it link `k256`; see the note on the
/// `tinywallet-bus` dependency.
///
/// The phrase is only sent to a module tinybus has attested *and* whose digest
/// matches one of the entries below — `super::wallet::attested_proxy` checks
/// this table itself rather than trusting that some check happened.
///
/// The contract also exposes `ExportKey` for downstream hosts that must drive
/// a signer locally; OpenHuman itself does not call it.
///
/// Three releases got here, and the order mattered. v0.2.3 changed no method at
/// all — it was the same module rebuilt against a bus that could attest it.
/// Attestation used to be recorded only from a `modules.toml` beside the
/// artifact, and a release download extracts into a temporary directory that has
/// none, so this module could never be an attested recipient however carefully
/// the digest below was pinned (tinybus#15 fixed that). Only then was it safe
/// for v0.3.0 to add methods that take a secret, and for v0.4.0 to add
/// `SignMessage` for the Solana and x402 encodings the wire contract does not
/// model. Adding them earlier would have made them unreachable in production and
/// reachable in a developer's tree, which is the worst of both.
pub(crate) const TINYWALLET: ModuleRecord = ModuleRecord {
    id: "tinywallet",
    description: "Transaction building and assembly for Bitcoin, EVM, Solana and Tron",
    bus_name: "ai.tinyhumans.tinywallet.Wallet",
    object_path: "/ai/tinyhumans/tinywallet/Wallet",
    version: "0.7.3",
    release_url: "https://github.com/tinyhumansai/tinywallet/releases/tag/v0.7.3",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinywallet-module-0.7.3-ubuntu-24.04-x86_64.tar.gz",
            sha256: "c5388860a63806ab8f27790760c992e3f0684c9eea884f7a54d0eedc2244a227",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinywallet-module-0.7.3-ubuntu-24.04-arm64.tar.gz",
            sha256: "064ada10d6045610ad7e52414d00f6bf6b53ea63bdfd7cc54ae4f2be95933949",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinywallet-module-0.7.3-ubuntu-22.04-x86_64.tar.gz",
            sha256: "cd647c436d2a991c20758db221fbb986c5edf33aabcaafb282701f590367c171",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinywallet-module-0.7.3-ubuntu-22.04-arm64.tar.gz",
            sha256: "635ba9b9b1e1d555465c9f8d30f658c1d9377fa20042139656175a08780a4edc",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinywallet-module-0.7.3-macos-26-arm64.tar.gz",
            sha256: "c0c09dae79c10b059e5774de298431506cf599bb5154a98ccd8cd3a06888d3c7",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinywallet-module-0.7.3-macos-26-x86_64.tar.gz",
            sha256: "5bc6a2b85e7965358c6fa677feef047e59904809552eccde234604df297c7784",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinywallet-module-0.7.3-macos-15-arm64.tar.gz",
            sha256: "b3f1c7c2041b277d40ec4c4c5014b78f1a1aaa9d433a6f88cd179822214f3ddd",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinywallet-module-0.7.3-macos-15-x86_64.tar.gz",
            sha256: "b1d4acd71ebadb75c4cf32d1ecd3b349e8f70183edcded771a92ce3e776c0455",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinywallet-module-0.7.3-windows-2025-x86_64.zip",
            sha256: "4fec78a39b4875cd1a17977fa454491c68d5c1fd6fc50ffc2b985caec48b9151",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinywallet-module-0.7.3-windows-2022-x86_64.zip",
            sha256: "bbb0e04219c38909e79d340edc57019c4b0be5989f372bcbb5316da1f5311f79",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinywallet-module-0.7.3-windows-11-arm64.zip",
            sha256: "54c516a7d36b7ef800f15c4c98815f895c1be76f6109b4dce4ff779452bc1f1c",
        },
    ],
    load: LoadPolicy::Lazy,
};
