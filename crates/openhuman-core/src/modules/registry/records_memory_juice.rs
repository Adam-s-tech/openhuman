//! Registry records for the `tinymemory` and `tinyjuice` modules.

use crate::modules::types::{LoadPolicy, ModuleRecord, PlatformAsset};

/// The complete TinyMemory engine, loaded eagerly so its capabilities are
/// available when the kernel assembles its RPC and tool surfaces.
pub(crate) const TINYMEMORY: ModuleRecord = ModuleRecord {
    id: "tinymemory",
    description: "Local memory engine: store, ranked recall, and portable export",
    bus_name: "ai.tinyhumans.tinymemory.Memory",
    object_path: "/ai/tinyhumans/tinymemory/Memory",
    version: "1.20.0",
    release_url: "https://github.com/tinyhumansai/tinymemory/releases/tag/v1.20.0",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinymemory-module-1.20.0-ubuntu-24.04-x86_64.tar.gz",
            sha256: "f2e2355098e358388fc59525ae796a2ee221aea26805fa8fc9870962067cc945",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinymemory-module-1.20.0-ubuntu-24.04-arm64.tar.gz",
            sha256: "e02423896248f374c0c1fab7a0868449e046238a6e9b0001e9ff7e46799aea77",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinymemory-module-1.20.0-ubuntu-22.04-x86_64.tar.gz",
            sha256: "d3c2485a3fdd657b40ba65ab266950bd3f3f060695a9bba3e20b2d83e343fb61",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinymemory-module-1.20.0-ubuntu-22.04-arm64.tar.gz",
            sha256: "0a8a54ce6569782d1852c9e8ffefd0e6bb3c35c30ecb9357642cf94bcee5d81f",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinymemory-module-1.20.0-macos-26-arm64.tar.gz",
            sha256: "f62caa85b7e091714e8abcc4f5bcbe47265518f058a7d66326049c64fd209d5b",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinymemory-module-1.20.0-macos-26-x86_64.tar.gz",
            sha256: "b13d93054145a3a0159ff4df61bd837cfb6cd2aec1e7631fff0d937b050acf8c",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinymemory-module-1.20.0-macos-15-arm64.tar.gz",
            sha256: "94b7bcaa893fb52a690ce870139e8387636a23c2ab180afbae2a39049687cf69",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinymemory-module-1.20.0-macos-15-x86_64.tar.gz",
            sha256: "cf7073d557d7b9b85c2f3face27584101904675d0b5728cabf54ab03c46a3de8",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinymemory-module-1.20.0-windows-2025-x86_64.zip",
            sha256: "22a38f8e43c734f355636a3e36c710f72fa9f1611232ee6197f2a3c800bf7f7f",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinymemory-module-1.20.0-windows-2022-x86_64.zip",
            sha256: "36579172066bcee245bfd5f94fcf6523f892d1daf6f219dea8a50879094c8ccc",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinymemory-module-1.20.0-windows-11-arm64.zip",
            sha256: "5d016281aa6e719a5ae08e6482bc53e17527afba9e2d7c859f10a103dba68a0a",
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
