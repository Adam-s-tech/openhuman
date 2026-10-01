//! Registry records for the `tinymemory` and `tinyjuice` modules.

use crate::modules::types::{LoadPolicy, ModuleRecord, PlatformAsset};

/// The complete TinyMemory engine, loaded eagerly so its capabilities are
/// available when the kernel assembles its RPC and tool surfaces.
pub(crate) const TINYMEMORY: ModuleRecord = ModuleRecord {
    id: "tinymemory",
    description: "Local memory engine: store, ranked recall, and portable export",
    bus_name: "ai.tinyhumans.tinymemory.Memory",
    object_path: "/ai/tinyhumans/tinymemory/Memory",
    version: "1.21.0",
    release_url: "https://github.com/tinyhumansai/tinymemory/releases/tag/v1.21.0",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinymemory-module-1.21.0-ubuntu-24.04-x86_64.tar.gz",
            sha256: "6a64dedef6eff6b52008dea53c4f1a00fdd003a8036aa119c8263d9e95d7b40b",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinymemory-module-1.21.0-ubuntu-24.04-arm64.tar.gz",
            sha256: "777258db3fb8bcd067e5c1f07adf4d1c90c50c139727b8e37debada50bce39a5",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinymemory-module-1.21.0-ubuntu-22.04-x86_64.tar.gz",
            sha256: "600dbfba33f7f37d9eea41ae57ce60e5f76b3e38bb23a5b2b0e8c502259c9dd0",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinymemory-module-1.21.0-ubuntu-22.04-arm64.tar.gz",
            sha256: "f232e3bb27b008fadd27bfba072c93c1678b7a40377c3e8b323f2bd2d7ff0d77",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinymemory-module-1.21.0-macos-26-arm64.tar.gz",
            sha256: "704ba0fc18d6d4f3c3b7cc24122751dab8a5aff281b9b9812cbd5d588e030189",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinymemory-module-1.21.0-macos-26-x86_64.tar.gz",
            sha256: "5303b4c6dfbef88260a57dae4c11e5b8156f3f9e22b694bf08f7f051b070510f",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinymemory-module-1.21.0-macos-15-arm64.tar.gz",
            sha256: "a887c88a1de3e16976b1140799f14227b1fd29fec9efd6fc891d1c3bc49a0f0c",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinymemory-module-1.21.0-macos-15-x86_64.tar.gz",
            sha256: "7be5821777ab1d7c543cbc4606e88277470fe0d46ca9d7bc9e221d22a4ca7fdf",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinymemory-module-1.21.0-windows-2025-x86_64.zip",
            sha256: "bcccbd8f82f435281eb1716cf6dce5b5308a08f36b41bfd4c61657f6839d9365",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinymemory-module-1.21.0-windows-2022-x86_64.zip",
            sha256: "d2d4b22b346f08d535a6ea20ad24800371925c6f28c78aac46e84c43e573f477",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinymemory-module-1.21.0-windows-11-arm64.zip",
            sha256: "cf34a7d2937c831507076f8b7f20d9f08055777ec63f93a247a00c7717a63522",
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
