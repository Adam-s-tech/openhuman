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
    version: "0.6.0",
    release_url: "https://github.com/tinyhumansai/tinywallet/releases/tag/v0.6.0",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinywallet-module-0.6.0-ubuntu-24.04-x86_64.tar.gz",
            sha256: "b4cec52b97f33bd1293e5c54825f1976d816d0186c0802aaae5196b5699f98cb",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinywallet-module-0.6.0-ubuntu-24.04-arm64.tar.gz",
            sha256: "53c4ab2183df15983fcc51a1a608d19bf9db09e29a06f4ccac42c6f58ba046fd",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinywallet-module-0.6.0-ubuntu-22.04-x86_64.tar.gz",
            sha256: "cba53f33fb50045c999d0d4bb2656276557873732701ccc5e6e257431574a9db",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinywallet-module-0.6.0-ubuntu-22.04-arm64.tar.gz",
            sha256: "27e9c8b2ade63ba05626c2f37a24f039a80c1417e1df88f1335776bd1a438abf",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinywallet-module-0.6.0-macos-26-arm64.tar.gz",
            sha256: "861433f7d8d35f67b2494e603ce7b0d59e99ef573f143bcb24bf36b40c22d0a5",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinywallet-module-0.6.0-macos-26-x86_64.tar.gz",
            sha256: "0d23e2d4edc7e52d675ca97efcddfaa9d3cd4e2804194242ae95c29ebe3ee61d",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinywallet-module-0.6.0-macos-15-arm64.tar.gz",
            sha256: "6f2f401c7511220dab81f421f48e54d348da7d3bc980f1a5c316e9f0bc8fad6c",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinywallet-module-0.6.0-macos-15-x86_64.tar.gz",
            sha256: "df559e9305a336b94437efabf6ea882b3c65d3707dc5de204020da921f57427f",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinywallet-module-0.6.0-windows-2025-x86_64.zip",
            sha256: "47be0c22f4c15fc0fa38b3f770ab963d4b705b512e9be4baec542bea4bd150b5",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinywallet-module-0.6.0-windows-2022-x86_64.zip",
            sha256: "f4a44eb9803c0372ed621dd84dcafa752e671dabb7dabff7712a245fd16ab39a",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinywallet-module-0.6.0-windows-11-arm64.zip",
            sha256: "feafad7d71af77303185b52e0983c20424158b7ac3f24020c04a02c0b937537a",
        },
    ],
    load: LoadPolicy::Lazy,
};
