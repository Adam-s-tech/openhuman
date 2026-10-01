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
