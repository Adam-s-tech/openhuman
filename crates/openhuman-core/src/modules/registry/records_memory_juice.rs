//! Registry records for the `tinymemory` and `tinyjuice` modules.

use crate::modules::types::{LoadPolicy, ModuleRecord, PlatformAsset};

/// The complete TinyMemory engine, loaded eagerly so its capabilities are
/// available when the kernel assembles its RPC and tool surfaces.
pub(crate) const TINYMEMORY: ModuleRecord = ModuleRecord {
    id: "tinymemory",
    description: "Local memory engine: store, ranked recall, and portable export",
    bus_name: "ai.tinyhumans.tinymemory.Memory",
    object_path: "/ai/tinyhumans/tinymemory/Memory",
    version: "1.17.0",
    release_url: "https://github.com/tinyhumansai/tinymemory/releases/tag/v1.17.0",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinymemory-module-1.17.0-ubuntu-24.04-x86_64.tar.gz",
            sha256: "c3da7f6a0bfae1224d8f3cda9382b6236f8118d60c96ce8b26fea8f16234ed08",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinymemory-module-1.17.0-ubuntu-24.04-arm64.tar.gz",
            sha256: "57cc138a6dcf77eeaed2f9f8d27182713966e855c5db66b80b5857a8ca8ff236",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinymemory-module-1.17.0-ubuntu-22.04-x86_64.tar.gz",
            sha256: "0b0305bddbee09b3fe9965395fe9d006526f0c0c2ffd777d6e78747b2cfcff35",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinymemory-module-1.17.0-ubuntu-22.04-arm64.tar.gz",
            sha256: "b6966f6c3d7c7d4b34f6b0c5d28077dee8809120326daa163b838b1d876c36d0",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinymemory-module-1.17.0-macos-26-arm64.tar.gz",
            sha256: "c1e39dcef00f37ce0a2ff708790b5de04814ce113cfa7f16bf190fcfaa49645e",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinymemory-module-1.17.0-macos-26-x86_64.tar.gz",
            sha256: "ef59b7c3549c6c718ba26871b9ab54322f248d9b0c8219b265b56d00f6e280a9",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinymemory-module-1.17.0-macos-15-arm64.tar.gz",
            sha256: "09a0a27e09e8986d3dbe1f5c32594796b12f59b7d7822afdd746c3514f704788",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinymemory-module-1.17.0-macos-15-x86_64.tar.gz",
            sha256: "31df68800cec34555a0432b1f057faf2e5cc74af95b43557349d1655d2825ce0",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinymemory-module-1.17.0-windows-2025-x86_64.zip",
            sha256: "120619d1f1efc4627d7f8418b69cde94c89274f20b4d4c3ed6bb5d9ed23637cf",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinymemory-module-1.17.0-windows-2022-x86_64.zip",
            sha256: "825938f178675ad488a7add4b4f4c78c0cf5107d589f05c1bfbbd459a9e94e33",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinymemory-module-1.17.0-windows-11-arm64.zip",
            sha256: "56cf207610e58cf70afed2752a2a76a16e2542d2255c0b1719d54f2a7373bf1d",
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
