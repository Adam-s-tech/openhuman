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
    version: "0.1.19",
    release_url: "https://github.com/tinyhumansai/tinydocs/releases/tag/v0.1.19",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinydocs-module-0.1.19-ubuntu-24.04-x86_64.tar.gz",
            sha256: "3d095f55b689cb323ae8e301beef9b9a9f16cddd3cd7c9182d346dd8e2b50f7c",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinydocs-module-0.1.19-ubuntu-24.04-arm64.tar.gz",
            sha256: "b61b2ec9f2f2d31733c3b428741eb1f32a05e0f45e0e23433d1ecc4db2701bfd",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinydocs-module-0.1.19-ubuntu-22.04-x86_64.tar.gz",
            sha256: "e5718a7ff0d82c9a017f733d2ae030f2092090ff285b010a502b96325e93d6d0",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinydocs-module-0.1.19-ubuntu-22.04-arm64.tar.gz",
            sha256: "a61fb3de13dfaa518ccbcbaa77e1483f72ed10d4f38c097b0b428d83a0223bbf",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinydocs-module-0.1.19-macos-26-arm64.tar.gz",
            sha256: "183dbcbc0aa2460e42cf7bda3c68435eaddefabfec0d46a33e7f25c1753488b3",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinydocs-module-0.1.19-macos-26-x86_64.tar.gz",
            sha256: "c58396122e46ce195c7edbfbe9c867098275080eef92c9b9beb8e234fe8934b6",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinydocs-module-0.1.19-macos-15-arm64.tar.gz",
            sha256: "05afa7caf2ee73923fc72ea9fe10e6b2290ec79c8affafbd851c03389e070dd0",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinydocs-module-0.1.19-macos-15-x86_64.tar.gz",
            sha256: "b2222b50c0cfd62e4f891d4204559c9bf9bfbb4276363e6a4f2ebe2973370bae",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinydocs-module-0.1.19-windows-2025-x86_64.zip",
            sha256: "3d9113595bf83b2e8c28f56f45c8ab3527de89a880b0a0319df6c19c8399cb47",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinydocs-module-0.1.19-windows-2022-x86_64.zip",
            sha256: "e25b2585a56f68fd86db45f072743bc090a664ca584470c22b9f7221498e6533",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinydocs-module-0.1.19-windows-11-arm64.zip",
            sha256: "e1ffd5314571070236c1fc52dca490151072473b40ba5a86877579efa3351209",
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
    version: "0.7.2",
    release_url: "https://github.com/tinyhumansai/tinywallet/releases/tag/v0.7.2",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinywallet-module-0.7.2-ubuntu-24.04-x86_64.tar.gz",
            sha256: "63b3905992f49c35b68b0a0e3e81a324ce8ea0e8febc77461083930da5064a4c",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinywallet-module-0.7.2-ubuntu-24.04-arm64.tar.gz",
            sha256: "dc56c04e006771e0f10a901dce34fdbc2a930650c22a06983c695b90997ce37a",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinywallet-module-0.7.2-ubuntu-22.04-x86_64.tar.gz",
            sha256: "a429f3fc617735d50de5f723c79a7ebd8b50948fd3ddb7294e4312891ea5d43f",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinywallet-module-0.7.2-ubuntu-22.04-arm64.tar.gz",
            sha256: "edc9f7649c367a5197f288028dcd1e66b018d8589a80f96b29e1aae0d833d4b0",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinywallet-module-0.7.2-macos-26-arm64.tar.gz",
            sha256: "3699f7769b6c2c4804b7b65fee5251be9e4dff25a5f8b711eb040906cbe504f3",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinywallet-module-0.7.2-macos-26-x86_64.tar.gz",
            sha256: "41341afe77e478282b90a54280e6882c11bdc78c04a2c629f1904ca2e568617e",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinywallet-module-0.7.2-macos-15-arm64.tar.gz",
            sha256: "599395b9ffe27ac14c018ac0b0de03e73095f7ee8415b8132507d6e5ea5eab76",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinywallet-module-0.7.2-macos-15-x86_64.tar.gz",
            sha256: "dcff631541c527a00f2cdef0cb811f65e9c442d23656692bdd6c5c1e57b459ab",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinywallet-module-0.7.2-windows-2025-x86_64.zip",
            sha256: "09ccf416eb2683d36bede7b0b89ea08b0e0456f6d4bc2374fc8c23436737661f",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinywallet-module-0.7.2-windows-2022-x86_64.zip",
            sha256: "ebd9dcedd60b55caad3e4d723ef0448782f0f18ac8beebd8a6db99dbb13e3ce2",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinywallet-module-0.7.2-windows-11-arm64.zip",
            sha256: "b6be18cacfaf001fd378131a1374fb9332c1adaee18ea8180841bda718184ea9",
        },
    ],
    load: LoadPolicy::Lazy,
};
