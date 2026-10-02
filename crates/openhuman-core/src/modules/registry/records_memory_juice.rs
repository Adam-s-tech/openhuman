//! Registry records for the `tinymemory` and `tinyjuice` modules.

use crate::modules::types::{LoadPolicy, ModuleRecord, PlatformAsset};

/// The complete TinyMemory engine, loaded eagerly so its capabilities are
/// available when the kernel assembles its RPC and tool surfaces.
pub(crate) const TINYMEMORY: ModuleRecord = ModuleRecord {
    id: "tinymemory",
    description: "Local memory engine: store, ranked recall, and portable export",
    bus_name: "ai.tinyhumans.tinymemory.Memory",
    object_path: "/ai/tinyhumans/tinymemory/Memory",
    version: "1.22.3",
    release_url: "https://github.com/tinyhumansai/tinymemory/releases/tag/v1.22.3",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinymemory-module-1.22.3-ubuntu-24.04-x86_64.tar.gz",
            sha256: "f305e12e09a26329192cb83f83755534c13d60bf1a4123efa280af8030b73dc6",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinymemory-module-1.22.3-ubuntu-24.04-arm64.tar.gz",
            sha256: "adea2d73604b4d984e23ed4250dcf8ca70e7c72b215652362bbd9059bd6edcbf",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinymemory-module-1.22.3-ubuntu-22.04-x86_64.tar.gz",
            sha256: "793b45d75f2055eb618784d74e7f9eb599d25e7018c5b4bce8f90fe7e51fd21b",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinymemory-module-1.22.3-ubuntu-22.04-arm64.tar.gz",
            sha256: "3e79cace52fcd339e73af5c448a505cf217184cefece7e5e7581ee34b1e80c77",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinymemory-module-1.22.3-macos-26-arm64.tar.gz",
            sha256: "fe3a4458cdd162450a3b005856fb0d31efd248c90d80d38f656d23b6fe54d814",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinymemory-module-1.22.3-macos-26-x86_64.tar.gz",
            sha256: "d9a3ed0ab01bad977e9b9b97349e95a012055dc8c52d795a5cf1cf0228da1306",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinymemory-module-1.22.3-macos-15-arm64.tar.gz",
            sha256: "7497cc8c1e38cae00bb6f7a14bfebe7b261057edc56e5488b3a3d1c079e3302d",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinymemory-module-1.22.3-macos-15-x86_64.tar.gz",
            sha256: "3b20c547537a50df0a8526e993bbb23f068559b26d21e4da9cad00d9b2d8b698",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinymemory-module-1.22.3-windows-2025-x86_64.zip",
            sha256: "b4fd3aef4ccda570abfb60692c16099fb168ab4caaa08bfaadde6fe11aafd044",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinymemory-module-1.22.3-windows-2022-x86_64.zip",
            sha256: "7cf5050ed841c6457f98023b9c64f6ffd18c1e9d28b8e3f83d49db63ac9fb2d5",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinymemory-module-1.22.3-windows-11-arm64.zip",
            sha256: "38e92ad03d7b26d9627856e160c99890878229aa4bf92be0418b529e5e2b64e3",
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
