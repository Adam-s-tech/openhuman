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
    version: "0.1.10",
    release_url: "https://github.com/tinyhumansai/tinyvoice/releases/tag/v0.1.10",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinyvoice-module-0.1.10-ubuntu-24.04-x86_64.tar.gz",
            sha256: "fb940e440d2f424a857b765d2665572a5f801ac2d09a518c3764f2eedc4b1e55",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinyvoice-module-0.1.10-ubuntu-24.04-arm64.tar.gz",
            sha256: "bb418c1feddaf931e94d51f8d168941b51f954cc29a704b3ffe644b9878f32e4",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinyvoice-module-0.1.10-ubuntu-22.04-x86_64.tar.gz",
            sha256: "f50587ae47338622245b75fcddcc4b1dc091b8a10eb1f89509987b464b5fce93",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinyvoice-module-0.1.10-ubuntu-22.04-arm64.tar.gz",
            sha256: "f8312808d05aaacbdfd260784c787511cc6e0714ec18cc88940d886092bda434",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinyvoice-module-0.1.10-macos-26-arm64.tar.gz",
            sha256: "655bfd8e17acb04a7feae3298f9bd7a8e30d35c3689d73ec7d42112d873f55e0",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinyvoice-module-0.1.10-macos-26-x86_64.tar.gz",
            sha256: "f527352e859fb15106b4d54d215f305e7325a683095a35972860dea7285c29b0",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinyvoice-module-0.1.10-macos-15-arm64.tar.gz",
            sha256: "b2abbd587e000f07160db204d8620732853af3147f6ad26983ad978ae545cb60",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinyvoice-module-0.1.10-macos-15-x86_64.tar.gz",
            sha256: "ff55f86fb49037541acfa598d220fc216ab75fab57586f77ef062e6a1360bc30",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinyvoice-module-0.1.10-windows-2025-x86_64.zip",
            sha256: "4f9e2366eed335e09374bc30985c26bac9334d47e792719408b5ce36c0964056",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinyvoice-module-0.1.10-windows-2022-x86_64.zip",
            sha256: "270a8ea2d03a7f0bff77e63a712b7cfd6db33c0232a7d7c0a4011ea97518bf45",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinyvoice-module-0.1.10-windows-11-arm64.zip",
            sha256: "5cbcc37f4d62b906719ad225569ae8098cb48da33922bf82b6eed33c6eb98cea",
        },
    ],
    load: LoadPolicy::Lazy,
};
