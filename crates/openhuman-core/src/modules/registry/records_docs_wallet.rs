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
    version: "0.1.18",
    release_url: "https://github.com/tinyhumansai/tinydocs/releases/tag/v0.1.18",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinydocs-module-0.1.18-ubuntu-24.04-x86_64.tar.gz",
            sha256: "40912f01518f386b0722b26d0e29b751790b7e11210cee3914e98fbc4948d6b1",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinydocs-module-0.1.18-ubuntu-24.04-arm64.tar.gz",
            sha256: "aaaa7a758d7a3368d3c1c3684a0e3bf1b421715e584ac3f7e00e27ce2185ac9e",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinydocs-module-0.1.18-ubuntu-22.04-x86_64.tar.gz",
            sha256: "48dfb265d867f0aaf7208c08415a18b58733384f5a8eb28c993acebf7a0cf8c8",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinydocs-module-0.1.18-ubuntu-22.04-arm64.tar.gz",
            sha256: "672847891d717009999d8cec14fe51be5248d524f90dde88a38431a0dcdd2386",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinydocs-module-0.1.18-macos-26-arm64.tar.gz",
            sha256: "bbb76a1ae7cc524340eaa8fc56c7ba8d286f7cd8179e1e39091776a17d31219f",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinydocs-module-0.1.18-macos-26-x86_64.tar.gz",
            sha256: "c0571422591dee71c040de736917e1559a8fd5c30825f00dcaa8ea8dfc90f3d9",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinydocs-module-0.1.18-macos-15-arm64.tar.gz",
            sha256: "ff214ad3e7150b729f8be499b413740c72947ef8c761070633909aedf4531627",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinydocs-module-0.1.18-macos-15-x86_64.tar.gz",
            sha256: "3d3af00948329fecbe016453644bd2c73f293d297d23ac2c77bdd2288bd3d708",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinydocs-module-0.1.18-windows-2025-x86_64.zip",
            sha256: "513d6e0e458f25f5bb70b5df3bbca64961db7d845eb9ca2a3b28150c46703791",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinydocs-module-0.1.18-windows-2022-x86_64.zip",
            sha256: "97530e3ded5b1372b67a9716b5421bfafa05f8bd118d35f85583c9260f794198",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinydocs-module-0.1.18-windows-11-arm64.zip",
            sha256: "a9683fc5bc2b05b194e3a540fddf5dd8931a8c5bba1d58c1cb484ba97d59079c",
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
    version: "0.7.1",
    release_url: "https://github.com/tinyhumansai/tinywallet/releases/tag/v0.7.1",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinywallet-module-0.7.1-ubuntu-24.04-x86_64.tar.gz",
            sha256: "1a290dc93523b5022e4332339945ce297649211f4cb6ac38ef14417326403b1e",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinywallet-module-0.7.1-ubuntu-24.04-arm64.tar.gz",
            sha256: "de3ab9473daa1f87ce3a1adefae150dd59b2b048daaf18a9388811652c3cdee1",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinywallet-module-0.7.1-ubuntu-22.04-x86_64.tar.gz",
            sha256: "9b279e76fcde4714316083aaa9dd3ad6c1e8bc68ecc230589bad4d3bd544a9bd",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinywallet-module-0.7.1-ubuntu-22.04-arm64.tar.gz",
            sha256: "c39cbea86a5e0237561ad4dfa3ca2e60bf6dafeedbe13fda422f2d680a1bbf5a",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinywallet-module-0.7.1-macos-26-arm64.tar.gz",
            sha256: "3df50f760368815ab07c18286d74e4c74216510685df93f4e7e5a4a01f458561",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinywallet-module-0.7.1-macos-26-x86_64.tar.gz",
            sha256: "47a90b9d0165af16f28502df7bd81330836db6a3df7952cf9a47a794f6923776",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinywallet-module-0.7.1-macos-15-arm64.tar.gz",
            sha256: "0b65ddde022b39ff94df7e651f91feaac6c3fcabf8373f0376ae5992e3d8952b",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinywallet-module-0.7.1-macos-15-x86_64.tar.gz",
            sha256: "aad179f3094b65d0370ac525b2978709e9242ba86569e2c6f962611ca513a31f",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinywallet-module-0.7.1-windows-2025-x86_64.zip",
            sha256: "00417bf719709512964d495c52d1d0c17a0b77fbf89d5841b0e2ca8df501af55",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinywallet-module-0.7.1-windows-2022-x86_64.zip",
            sha256: "e7e1e86a685bafa48bf404f815f99f13ce39272878ee32749ceecb38e1ba1529",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinywallet-module-0.7.1-windows-11-arm64.zip",
            sha256: "a484570cb5059806305745823627fd9d74b239091924ca254559befec1721447",
        },
    ],
    load: LoadPolicy::Lazy,
};
