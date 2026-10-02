//! Registry records for the `tinymemory` and `tinyjuice` modules.

use crate::modules::types::{LoadPolicy, ModuleRecord, PlatformAsset};

/// The complete TinyMemory engine, loaded eagerly so its capabilities are
/// available when the kernel assembles its RPC and tool surfaces.
pub(crate) const TINYMEMORY: ModuleRecord = ModuleRecord {
    id: "tinymemory",
    description: "Local memory engine: store, ranked recall, and portable export",
    bus_name: "ai.tinyhumans.tinymemory.Memory",
    object_path: "/ai/tinyhumans/tinymemory/Memory",
    version: "1.22.4",
    release_url: "https://github.com/tinyhumansai/tinymemory/releases/tag/v1.22.4",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinymemory-module-1.22.4-ubuntu-24.04-x86_64.tar.gz",
            sha256: "ebbdccbb4531c93dfc69d58ac0a2b665fc478f12d1a7cac2e80c4370b71cbbf7",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinymemory-module-1.22.4-ubuntu-24.04-arm64.tar.gz",
            sha256: "f2564bf18b981b860e35bdfc5e69d98b289376b922503888ad42d06221705e43",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinymemory-module-1.22.4-ubuntu-22.04-x86_64.tar.gz",
            sha256: "52b364047b1a7aaf287d5bb17840fcda20547286f3db98211bfe1d9962518ee9",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinymemory-module-1.22.4-ubuntu-22.04-arm64.tar.gz",
            sha256: "cedeced068759e8e629a2f13870428c86dd97026d995503d76c816fb8c4a5b83",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinymemory-module-1.22.4-macos-26-arm64.tar.gz",
            sha256: "dd3e9c1fd037585ecd731cccb877bbec8ffaa2a70b17cab45c4247fbf23d4263",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinymemory-module-1.22.4-macos-26-x86_64.tar.gz",
            sha256: "56615e7b735946195e4f3928fb901945f2ab124693b4fd3c5d4075ef00bc8c37",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinymemory-module-1.22.4-macos-15-arm64.tar.gz",
            sha256: "e666308b106575149ba6c51fe4ea3c2c8c65fcf3a57ae130de4c60ad6bb7e0eb",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinymemory-module-1.22.4-macos-15-x86_64.tar.gz",
            sha256: "7d64cc160c8fb7e0629225892013cf3733a34ecacd47eceae27651779fd965ed",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinymemory-module-1.22.4-windows-2025-x86_64.zip",
            sha256: "fe1a82c326d4fe0f227884575a7672b964c0f1c701d4b124319465a63a977162",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinymemory-module-1.22.4-windows-2022-x86_64.zip",
            sha256: "00f3789fc5bfd7b083ef38cee07a51ce6870e5604c13d4bd092598cf97856f05",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinymemory-module-1.22.4-windows-11-arm64.zip",
            sha256: "6f08653c131c4a8786f45e49ecbd9a4329a51b0505c121a7252c2fe27fbfe6a3",
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
