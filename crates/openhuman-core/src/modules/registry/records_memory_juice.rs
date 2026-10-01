//! Registry records for the `tinymemory` and `tinyjuice` modules.

use crate::modules::types::{LoadPolicy, ModuleRecord, PlatformAsset};

/// The complete TinyMemory engine, loaded eagerly so its capabilities are
/// available when the kernel assembles its RPC and tool surfaces.
pub(crate) const TINYMEMORY: ModuleRecord = ModuleRecord {
    id: "tinymemory",
    description: "Local memory engine: store, ranked recall, and portable export",
    bus_name: "ai.tinyhumans.tinymemory.Memory",
    object_path: "/ai/tinyhumans/tinymemory/Memory",
    version: "1.22.0",
    release_url: "https://github.com/tinyhumansai/tinymemory/releases/tag/v1.22.0",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinymemory-module-1.22.0-ubuntu-24.04-x86_64.tar.gz",
            sha256: "4b5e0d00a6618b30945e66033ed8ed593b78e5db666a21704cd350c0bef0be0f",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinymemory-module-1.22.0-ubuntu-24.04-arm64.tar.gz",
            sha256: "480997ccf1e198b4f0fa3b456dedf13c7a254ade25c91ac0d545324c0e2b2b75",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinymemory-module-1.22.0-ubuntu-22.04-x86_64.tar.gz",
            sha256: "a2dc684bc0398831231fc2bc7fe27831b4399c081dad700efddfa8a05e0013f1",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinymemory-module-1.22.0-ubuntu-22.04-arm64.tar.gz",
            sha256: "6bf11d3f77ff012793fc3b85af66579597efd11231e15d50bf0d28051722b07d",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinymemory-module-1.22.0-macos-26-arm64.tar.gz",
            sha256: "4df0eb16fada322e6e2e0c9c036e71682f685412922792197d908632c87684a0",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinymemory-module-1.22.0-macos-26-x86_64.tar.gz",
            sha256: "fb85e2b5ad310b6f327865500bc8544b99cfbdc4ee888d3aed3692c0a3882af1",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinymemory-module-1.22.0-macos-15-arm64.tar.gz",
            sha256: "a43a0cca4ade658662229c579c1a611104daaac460b7bcb1b693d03ec89ebd85",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinymemory-module-1.22.0-macos-15-x86_64.tar.gz",
            sha256: "8d451be3690d3b444c268c8acb5b6b762f251c2348a3e11c30092901255df242",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinymemory-module-1.22.0-windows-2025-x86_64.zip",
            sha256: "d2917ff5f8d606ddafba8cbdd835bb64355d5039615cf70d84a3c8ca695bc5c5",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinymemory-module-1.22.0-windows-2022-x86_64.zip",
            sha256: "a2ed8c7e89cde4fbad3e1b2805ad91a4003d91585cd6328b6eda115630482d67",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinymemory-module-1.22.0-windows-11-arm64.zip",
            sha256: "69901d41d30911cbca0aaa176c8270ca400272ea9f7a41ebf4a250eb5ad6ba03",
        },
    ],
    // Eager, unlike the two codecs above. A codec that is never asked for should
    // not be paid for, but a memory driver's absence changes what the kernel
    // offers rather than merely delaying it: capabilities are read at bind time
    // and the RPC surface and agent-tool list are filtered from them. Resolving
    // that during a user's first recall would mean the first recall is the one
    // that behaves differently.
    load: LoadPolicy::Eager,
};

/// The `tinyjuice` content-aware tool-output compression engine.
///
/// Lazy because the host's compaction policy can disable it, and a session that
/// never produces compressible tool output should not pay the download or
/// resident native-library cost.
pub(crate) const TINYJUICE: ModuleRecord = ModuleRecord {
    id: "tinyjuice",
    description: "Content-aware tool-output compression and recoverable caching",
    bus_name: "ai.tinyhumans.tinyjuice.Compression",
    object_path: "/ai/tinyhumans/tinyjuice/Compression",
    version: "0.5.0",
    release_url: "https://github.com/tinyhumansai/tinyjuice/releases/tag/v0.5.0",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinyjuice-module-0.5.0-ubuntu-24.04-x86_64.tar.gz",
            sha256: "1ea4b52fbf420724759190b047e44a1c28b78d7e0e6d890c32bff81fd0fee70f",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinyjuice-module-0.5.0-ubuntu-24.04-arm64.tar.gz",
            sha256: "057b5e1cc4ff21592065ad7b367e535b10caebe9520b83cc6949d30a6256024f",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinyjuice-module-0.5.0-ubuntu-22.04-x86_64.tar.gz",
            sha256: "94c8998c830d9454560c00055187198a598eefaf5be51bcecdcd194a66f3b388",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinyjuice-module-0.5.0-ubuntu-22.04-arm64.tar.gz",
            sha256: "7e1b47a96386e9df8185763abb296fc0b08d4bc943e45098370b00f0693902f6",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinyjuice-module-0.5.0-macos-26-arm64.tar.gz",
            sha256: "018de974d6846dac5916bb3bb4a7b8cde193515191efebcf4c92986d91e3c500",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinyjuice-module-0.5.0-macos-26-x86_64.tar.gz",
            sha256: "27472f640931f64344deec69e6a9a94723790c10b17a01578dee9aec6bb1662d",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinyjuice-module-0.5.0-macos-15-arm64.tar.gz",
            sha256: "b4f6c367f61e7e4c33ad2241487ee443f838ea3a724170e497102c223fb13395",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinyjuice-module-0.5.0-macos-15-x86_64.tar.gz",
            sha256: "35c6e8c94399ef7f9fd268022cc60d1a2bb9f2c79649d7a2204f8b8ce0cb5fc1",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinyjuice-module-0.5.0-windows-2025-x86_64.zip",
            sha256: "0806bd8741bda3b5510896261858d2bcca807710823ce51321cba55776923615",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinyjuice-module-0.5.0-windows-2022-x86_64.zip",
            sha256: "6ec4f6066b9bd082bc2eb57d3bef9dda06c5b50e8d10a1535042024fe09aae78",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinyjuice-module-0.5.0-windows-11-arm64.zip",
            sha256: "d609425636324061a27ac8763df70221a0d0d785ac90d403c8e4b1d3da1c2912",
        },
    ],
    load: LoadPolicy::Lazy,
};
