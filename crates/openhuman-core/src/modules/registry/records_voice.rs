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
    version: "0.1.8",
    release_url: "https://github.com/tinyhumansai/tinyvoice/releases/tag/v0.1.8",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinyvoice-module-0.1.8-ubuntu-24.04-x86_64.tar.gz",
            sha256: "84404653e1bbe3e90e9744f5670c9ba1b551b8fed747d2cf763b82091fee3288",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinyvoice-module-0.1.8-ubuntu-24.04-arm64.tar.gz",
            sha256: "13becfc66a47d22f728cb37319a8a8c6ea73931b30606273f5b37421604697ab",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinyvoice-module-0.1.8-ubuntu-22.04-x86_64.tar.gz",
            sha256: "1645c51f7c4edb1a28534a1d848e8e39f2b1fbb8eacc36c4247b37c4da5e8b6d",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinyvoice-module-0.1.8-ubuntu-22.04-arm64.tar.gz",
            sha256: "b560f73d80c5de6002a229eb2f77972bcc2ac19a322234980f088db1fa746e37",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinyvoice-module-0.1.8-macos-26-arm64.tar.gz",
            sha256: "5fd62df5155af97073f5d7231ac92698e064a456073f7a10bbc76fcd7fce9dfc",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinyvoice-module-0.1.8-macos-26-x86_64.tar.gz",
            sha256: "054baf95018adeeb1a348e11ab67d7c6b2635968fa1c3e71b205fdb80ca1d845",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinyvoice-module-0.1.8-macos-15-arm64.tar.gz",
            sha256: "6c37dcf6d87988a648d7794fa8281a2688171ccc887a9d81f1a410d7eace2375",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinyvoice-module-0.1.8-macos-15-x86_64.tar.gz",
            sha256: "6e331bc92a200cd148440bdaef6ee9e9ffecc05e7bf58d6895648c9ce1607387",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinyvoice-module-0.1.8-windows-2025-x86_64.zip",
            sha256: "62c1778e09810e635fff792fe81f247fc0364838439ce68d35767d55947ed151",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinyvoice-module-0.1.8-windows-2022-x86_64.zip",
            sha256: "104c8643fc5a9d3c9d1311699c27d48b082a9b36d2ef3b98fd56c81af2a2541c",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinyvoice-module-0.1.8-windows-11-arm64.zip",
            sha256: "c572ac2cc865030740bf1bc108b846afca77715e1449a1f64c5f749f51e82ae5",
        },
    ],
    load: LoadPolicy::Lazy,
};
