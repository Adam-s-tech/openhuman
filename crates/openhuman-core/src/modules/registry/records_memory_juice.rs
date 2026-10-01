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
    version: "0.4.0",
    release_url: "https://github.com/tinyhumansai/tinyjuice/releases/tag/v0.4.0",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinyjuice-module-0.4.0-ubuntu-24.04-x86_64.tar.gz",
            sha256: "5bdfef23171904a7cf7b2f86e1fdda713cdf42982478d2b4aa4b2b5cdd5e10eb",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinyjuice-module-0.4.0-ubuntu-24.04-arm64.tar.gz",
            sha256: "b04fb8c44dca621f0640500b1b5528a9809b840e0706600bab8ba50e58011d5d",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinyjuice-module-0.4.0-ubuntu-22.04-x86_64.tar.gz",
            sha256: "c404fa25c54c6e3888fdbdd971be9055f1e46e947c854f1556069a4e0d0390da",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinyjuice-module-0.4.0-ubuntu-22.04-arm64.tar.gz",
            sha256: "050d658eafe8acdcfcdc914d12547c12bceb321658e5dabb1ee73ac131c2d74a",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinyjuice-module-0.4.0-macos-26-arm64.tar.gz",
            sha256: "d4914b1a7bca8d7334aeb0ba1962287d49782fc594685dbc0872c87401e6779e",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinyjuice-module-0.4.0-macos-26-x86_64.tar.gz",
            sha256: "e25cf52ca0f8778279f06a6af0cb12f78a8c6d99053739e6c8e8581bb1afedf5",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinyjuice-module-0.4.0-macos-15-arm64.tar.gz",
            sha256: "0ab7b56cdcb7c1b5c34955a2ee48257867fe58e5bd7f0ccb72b76142aec75653",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinyjuice-module-0.4.0-macos-15-x86_64.tar.gz",
            sha256: "c88186e0506fd25eaca06d17fa82ebed2a2e25cadebb10fb8921ee1cb26dd523",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinyjuice-module-0.4.0-windows-2025-x86_64.zip",
            sha256: "73a0b42509b9eb4ebead2aa1a477d83aca67f5b95a7cfcabade1723632d11b1b",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinyjuice-module-0.4.0-windows-2022-x86_64.zip",
            sha256: "aa96c13fa179896d58bae4006b5557d8efb50b6caabec2f5e8d15fd7e6e73d75",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinyjuice-module-0.4.0-windows-11-arm64.zip",
            sha256: "cc101496fce17f48556bd84973440e0a59e1d4a65fecf1abf43608d0a9b283e4",
        },
    ],
    load: LoadPolicy::Lazy,
};
