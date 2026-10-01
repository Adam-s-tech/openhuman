//! Registry records for the `tinymemory` and `tinyjuice` modules.

use crate::modules::types::{LoadPolicy, ModuleRecord, PlatformAsset};

/// The complete TinyMemory engine, loaded eagerly so its capabilities are
/// available when the kernel assembles its RPC and tool surfaces.
pub(crate) const TINYMEMORY: ModuleRecord = ModuleRecord {
    id: "tinymemory",
    description: "Local memory engine: store, ranked recall, and portable export",
    bus_name: "ai.tinyhumans.tinymemory.Memory",
    object_path: "/ai/tinyhumans/tinymemory/Memory",
    version: "1.21.1",
    release_url: "https://github.com/tinyhumansai/tinymemory/releases/tag/v1.21.1",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinymemory-module-1.21.1-ubuntu-24.04-x86_64.tar.gz",
            sha256: "c44e5f428ce7024e65e951041a7586c777b0a7a390dd757316f9db2f2d869c8b",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinymemory-module-1.21.1-ubuntu-24.04-arm64.tar.gz",
            sha256: "cb6476ea3337b240efc92718ad3747e4e4fa45afed4a50999cae9c95a5ffdfd9",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinymemory-module-1.21.1-ubuntu-22.04-x86_64.tar.gz",
            sha256: "f208b1bf05e52af4ef7a9902d656827c471aa0a50aa3015ddb0ef9058e524c53",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinymemory-module-1.21.1-ubuntu-22.04-arm64.tar.gz",
            sha256: "62cd46e2f038081c5d7ffd63ffe0482dcd92e54b7b8eae57c814aa614c00d65e",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinymemory-module-1.21.1-macos-26-arm64.tar.gz",
            sha256: "4537bc352d716e5f71e50c64dae4ce5c763f4109bd6cb4ad3d3cf9b2229e6c87",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinymemory-module-1.21.1-macos-26-x86_64.tar.gz",
            sha256: "c785386c02f6d1c4095718979d60a55705489ce988eee524cfa2be9190f7b829",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinymemory-module-1.21.1-macos-15-arm64.tar.gz",
            sha256: "45e40ff8a92b4b5a49c7ad294d3fa560f2a4798e86623d69cb494b624dbf99b0",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinymemory-module-1.21.1-macos-15-x86_64.tar.gz",
            sha256: "c0b08056b65430fdb6256c4fc260a445a40c33a96624a8202ac82f1c51e483ed",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinymemory-module-1.21.1-windows-2025-x86_64.zip",
            sha256: "61dc219dbb1872c9ecf4bba5908c8a596f057da5fe2c5374e42b40d66ce1534f",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinymemory-module-1.21.1-windows-2022-x86_64.zip",
            sha256: "a9540bdcaa60f17eca1066e7903412a27738fa804a91e302e81d8c69abdebdba",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinymemory-module-1.21.1-windows-11-arm64.zip",
            sha256: "ecf8a25b5991e223f7a7b8d4b4f822ed3f284b79022e7c7e38ffbc4b8f506840",
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
