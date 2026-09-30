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
    version: "0.1.17",
    release_url: "https://github.com/tinyhumansai/tinydocs/releases/tag/v0.1.17",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinydocs-module-0.1.17-ubuntu-24.04-x86_64.tar.gz",
            sha256: "1dc4b1d4d699deb8e09d2def23307a1626a0dcbf306a589aeb96df277ee0118f",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinydocs-module-0.1.17-ubuntu-24.04-arm64.tar.gz",
            sha256: "715c60f279d5a80871795f5dfcf2a8ff98a0e0d2285a28e5dadf7bf14baf4653",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinydocs-module-0.1.17-ubuntu-22.04-x86_64.tar.gz",
            sha256: "a7d60249aa12f2d7750918389124c2bb6b77189e311c09aac218494c48aefdb5",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinydocs-module-0.1.17-ubuntu-22.04-arm64.tar.gz",
            sha256: "7f3fd5d300f515d5971c0abc8cdde3f522674afa2814c2c02a9e1190b0fb906d",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinydocs-module-0.1.17-macos-26-arm64.tar.gz",
            sha256: "a35f9711da00cf87c2f86a6bdc94d7fedf529e8dae0a17e00af12beaa2196dd9",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinydocs-module-0.1.17-macos-26-x86_64.tar.gz",
            sha256: "1b5c36b1b81fc5a4a0260a700afc307cbbdd28f31083da51c5c9f5743200a630",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinydocs-module-0.1.17-macos-15-arm64.tar.gz",
            sha256: "43f6c6bfacab2f123a1dab346269c46383025b5de577d2b74961ebd5a5e34216",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinydocs-module-0.1.17-macos-15-x86_64.tar.gz",
            sha256: "59cbfc0a6156ecab49912d2b71e7c1958cfc700c7a156e83958001b63fca47e0",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinydocs-module-0.1.17-windows-2025-x86_64.zip",
            sha256: "1ace4a2ce788fa8162a7cec6cf99776da718e2da1774d91730622abb4d74cc5e",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinydocs-module-0.1.17-windows-2022-x86_64.zip",
            sha256: "ad22ca6003a0479479fba324948668fa1949b29e425da099c156befba96f848f",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinydocs-module-0.1.17-windows-11-arm64.zip",
            sha256: "8064f69d22bcc4f082e8e5c4f69cdbc8cb9046ed5234ece1a1a751050407d0f0",
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
    version: "0.7.0",
    release_url: "https://github.com/tinyhumansai/tinywallet/releases/tag/v0.7.0",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinywallet-module-0.7.0-ubuntu-24.04-x86_64.tar.gz",
            sha256: "988a995ca43c20e2297f42d5c227ab776840ec498818715997c87dab1355c6d7",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinywallet-module-0.7.0-ubuntu-24.04-arm64.tar.gz",
            sha256: "a57c39ca19b0c17021589e8565be707ad32193edfd0d70216a74b89150e4c44b",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinywallet-module-0.7.0-ubuntu-22.04-x86_64.tar.gz",
            sha256: "995d6ef4d5dba891768ffadee2aa0a8a0e91b6aa1015e4a7fa2f204fba6fb23e",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinywallet-module-0.7.0-ubuntu-22.04-arm64.tar.gz",
            sha256: "06a15fc4308098c948442d1d10f15537d58c0f42cad6100f1b89dbd15d3f026c",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinywallet-module-0.7.0-macos-26-arm64.tar.gz",
            sha256: "ee8a5c1b0c383f56af6d7774863c08708f45f968e757ce1fb829897e9347d771",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinywallet-module-0.7.0-macos-26-x86_64.tar.gz",
            sha256: "330f3071e035252454de920d58b4fad83791ad2132b0d60cf23c417b83674380",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinywallet-module-0.7.0-macos-15-arm64.tar.gz",
            sha256: "5831feb778e3792014e0456a370b65953ecbefa60947d451c51dec3ac278608f",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinywallet-module-0.7.0-macos-15-x86_64.tar.gz",
            sha256: "c85bb9b63001d50672748c6bb5731abf6fc013244b3d5a61e74eda93177cf74b",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinywallet-module-0.7.0-windows-2025-x86_64.zip",
            sha256: "1f5d8abcb198a3d12de02aa8df459422308a5b87a0cc93287f2c0fc7c3c4781e",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinywallet-module-0.7.0-windows-2022-x86_64.zip",
            sha256: "7e20c60ccfe8caf839bab11d8b7af040375bb04f1075526ca9b422374cf77500",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinywallet-module-0.7.0-windows-11-arm64.zip",
            sha256: "3104e7c403eea27b5ac6c3b60cdec5d2ac3ea19af3d40c41dba66003d1e6ae2b",
        },
    ],
    load: LoadPolicy::Lazy,
};
