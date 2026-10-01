//! Registry records for the `tinymemory` and `tinyjuice` modules.

use crate::modules::types::{LoadPolicy, ModuleRecord, PlatformAsset};

/// The complete TinyMemory engine, loaded eagerly so its capabilities are
/// available when the kernel assembles its RPC and tool surfaces.
pub(crate) const TINYMEMORY: ModuleRecord = ModuleRecord {
    id: "tinymemory",
    description: "Local memory engine: store, ranked recall, and portable export",
    bus_name: "ai.tinyhumans.tinymemory.Memory",
    object_path: "/ai/tinyhumans/tinymemory/Memory",
    version: "1.22.1",
    release_url: "https://github.com/tinyhumansai/tinymemory/releases/tag/v1.22.1",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinymemory-module-1.22.1-ubuntu-24.04-x86_64.tar.gz",
            sha256: "6ec90bc25b49c8e50d08185c461b4ab030b1dfb78b3610867349d81eee038d80",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinymemory-module-1.22.1-ubuntu-24.04-arm64.tar.gz",
            sha256: "3a0a7a7a7fcb2a3b8de7c94b2a51c445af6fd5e8c2248a800c583bdbe6a1b0d1",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinymemory-module-1.22.1-ubuntu-22.04-x86_64.tar.gz",
            sha256: "93071ee81e99502b25e293ed164c481126cb6e7bc4597e8c8e8c6964c90aa5f0",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinymemory-module-1.22.1-ubuntu-22.04-arm64.tar.gz",
            sha256: "45694ab10de1601f9a541e870d3aa964edb8d4ac81773fec3ada432f60a276ae",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinymemory-module-1.22.1-macos-26-arm64.tar.gz",
            sha256: "9ca9569bd8c54b51cce3db69b445c4d49c6f13fbca07cecf3e01680760a2b7ba",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinymemory-module-1.22.1-macos-26-x86_64.tar.gz",
            sha256: "87083d2bc19376348ef69332ae2866d9333e9430ebb5fac4695b2b83f72ead5d",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinymemory-module-1.22.1-macos-15-arm64.tar.gz",
            sha256: "f68ed24c57ef63ac336909de3c5537d386e5ed483636c462d2bbff25a22f9633",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinymemory-module-1.22.1-macos-15-x86_64.tar.gz",
            sha256: "8ef669b5e648c3a39466d3ce093d9354a38fc73aa6c762c637ecf184da2e2739",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinymemory-module-1.22.1-windows-2025-x86_64.zip",
            sha256: "776ff7f6ca5ee79cc18d81223f307bb14da10605c8b97ce2ed074da1fdc1be5a",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinymemory-module-1.22.1-windows-2022-x86_64.zip",
            sha256: "ac8b1a9e616c1f4ad957cc147d899c8530017665ee7a0b29da349b84380d3e35",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinymemory-module-1.22.1-windows-11-arm64.zip",
            sha256: "2db234800993473fbec9e51d3f488cd190d076f2350b0765ebab2ee2382b827c",
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
