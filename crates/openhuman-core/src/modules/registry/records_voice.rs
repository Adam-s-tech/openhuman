//! Registry record for the `tinyvoice` module.

use crate::modules::types::{LoadPolicy, ModuleRecord, PlatformAsset};

/// The `tinyvoice` module: the host-agnostic half of the voice pipeline.
///
/// Wake-word gating, fast-path command routing, STT hallucination detection,
/// and the capture-side audio work (downmix, resample, silence gate, WAV
/// framing).
///
/// Lazy, and more clearly so than the others: voice is opt-in twice over — a
/// user has to enable dictation or always-on listening before any of this runs
/// — so a session that never speaks should not pay a download or a `dlopen`.
///
/// **The VAD deliberately does not come through here.** A segmenter is driven
/// once per 20 ms frame from inside a `cpal` callback, and a bus round trip at
/// that cadence would cost more than the sixty-line state machine it replaces.
/// `voice::always_on` keeps its own; see [`super::voice`].
pub(crate) const TINYVOICE: ModuleRecord = ModuleRecord {
    id: "tinyvoice",
    description: "Wake-word gating, command routing, hallucination detection, capture audio",
    bus_name: "ai.tinyhumans.tinyvoice.Voice",
    object_path: "/ai/tinyhumans/tinyvoice/Voice",
    version: "0.1.9",
    release_url: "https://github.com/tinyhumansai/tinyvoice/releases/tag/v0.1.9",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinyvoice-module-0.1.9-ubuntu-24.04-x86_64.tar.gz",
            sha256: "57ef381ce3aae66c31d0a04fbc0e7fc1a7eae05f050df2f33fe1cb78641e1077",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinyvoice-module-0.1.9-ubuntu-24.04-arm64.tar.gz",
            sha256: "1e827772f11a58fae61d5c3f56f402517d01877353291c711fb391818b56334f",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinyvoice-module-0.1.9-ubuntu-22.04-x86_64.tar.gz",
            sha256: "96551874d10014e65d048ef638b797cb5ee2bc1d754a270b7042a168c4dea505",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinyvoice-module-0.1.9-ubuntu-22.04-arm64.tar.gz",
            sha256: "f0ac14a89b4bbbe324213a655838fc916c1c6a6cf0a5a83d6166de6d191a7918",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinyvoice-module-0.1.9-macos-26-arm64.tar.gz",
            sha256: "d207bb7b3cfbd23a58f18b73c05cbbf2c6451475b65a8d4e2e13463960db081e",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinyvoice-module-0.1.9-macos-26-x86_64.tar.gz",
            sha256: "298b6a4f33ac2103d4eed0efa3edc4ca3410364024bea84cac203c87bed20d99",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinyvoice-module-0.1.9-macos-15-arm64.tar.gz",
            sha256: "6d9c5afc7094a904099118839f4a745dfdc252006a93bac667c32c20955cac16",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinyvoice-module-0.1.9-macos-15-x86_64.tar.gz",
            sha256: "ceab1294c79ede3bca26415e202b897f11ea80be7c5f581fe57042b65b8a3332",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinyvoice-module-0.1.9-windows-2025-x86_64.zip",
            sha256: "172d63c964e4cddb7c80b93dadd43b78e405f649956294cc2b9dfa0606d425d1",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinyvoice-module-0.1.9-windows-2022-x86_64.zip",
            sha256: "54a760f81d7fd8ef9f0d4399f58b1793da57caadda3cc8054a20ac8bdb7fed38",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinyvoice-module-0.1.9-windows-11-arm64.zip",
            sha256: "cb65d043c9ed9f9d65ad72c9f6895f019ec290979e38cd98739f4e3174ffe057",
        },
    ],
    load: LoadPolicy::Lazy,
};
