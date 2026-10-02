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
    version: "0.5.1",
    release_url: "https://github.com/tinyhumansai/tinyjuice/releases/tag/v0.5.1",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinyjuice-module-0.5.1-ubuntu-24.04-x86_64.tar.gz",
            sha256: "0a9cefa18612452374d95ecd04c0f9924251711b735a1e5fb8b699b64b39dadc",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinyjuice-module-0.5.1-ubuntu-24.04-arm64.tar.gz",
            sha256: "e842b7fad7b0b3a9fdc375da096b8ed42035b8105c8ab461fcae22225314814b",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinyjuice-module-0.5.1-ubuntu-22.04-x86_64.tar.gz",
            sha256: "27e5ffe19ea712b1abb52c5d88fd20979c7106daef2dfe03e6d1fac782936da6",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinyjuice-module-0.5.1-ubuntu-22.04-arm64.tar.gz",
            sha256: "09d831e5b80bc6a8a9da8824597141ec6dbe97857d747aa75e33ca247215fdcf",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinyjuice-module-0.5.1-macos-26-arm64.tar.gz",
            sha256: "0408f261d644bf13ad51ea774e15a46c702c787e8ab45ca33918880250508332",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinyjuice-module-0.5.1-macos-26-x86_64.tar.gz",
            sha256: "95fe251a955fb3dc9ef6aee41910424a59299bf0be71f9b7a59dd3ea4bfd35ff",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinyjuice-module-0.5.1-macos-15-arm64.tar.gz",
            sha256: "09a4e971bc26a06060a4c4cba291b42ddf5c3898cf1a31ba6ba176f20d23577f",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinyjuice-module-0.5.1-macos-15-x86_64.tar.gz",
            sha256: "1205c5babeda78aaaf1e48a68779303f6863dd5fd09b0b85565b2d16d7b801c2",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinyjuice-module-0.5.1-windows-2025-x86_64.zip",
            sha256: "f19f2c6c9d61498ef5590c8c0ca24941daca3c3b88fee49169714ea61ba387be",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinyjuice-module-0.5.1-windows-2022-x86_64.zip",
            sha256: "684ae602ec587bff5947532a213fb9e132bd18c5af125e42016218aa096d25e8",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinyjuice-module-0.5.1-windows-11-arm64.zip",
            sha256: "b9b7a4776b04939ce2818591f0ad19f665d709aa62647e2e557924bcc81d07de",
        },
    ],
    load: LoadPolicy::Lazy,
};
