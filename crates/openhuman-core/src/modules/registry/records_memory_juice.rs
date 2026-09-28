//! Registry records for the `tinymemory` and `tinyjuice` modules.

use crate::modules::types::{LoadPolicy, ModuleRecord, PlatformAsset};

/// The complete TinyMemory engine, loaded eagerly so its capabilities are
/// available when the kernel assembles its RPC and tool surfaces.
pub(crate) const TINYMEMORY: ModuleRecord = ModuleRecord {
    id: "tinymemory",
    description: "Local memory engine: store, ranked recall, and portable export",
    bus_name: "ai.tinyhumans.tinymemory.Memory",
    object_path: "/ai/tinyhumans/tinymemory/Memory",
    version: "1.16.1",
    release_url: "https://github.com/tinyhumansai/tinymemory/releases/tag/v1.16.1",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinymemory-module-1.16.1-ubuntu-24.04-x86_64.tar.gz",
            sha256: "6afe17e3edd80e46538860e9445bfc45d205b8814587d49062117cb4da6edec8",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinymemory-module-1.16.1-ubuntu-24.04-arm64.tar.gz",
            sha256: "1348cd626b1ed109270ce801aeb4e68178d08daefaf79f142ccccc82bb9f1944",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinymemory-module-1.16.1-ubuntu-22.04-x86_64.tar.gz",
            sha256: "c025f4a5743bb975ed7cdc5306a16a6aa4ac2ef3c87fa83820d038321123daee",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinymemory-module-1.16.1-ubuntu-22.04-arm64.tar.gz",
            sha256: "27bc9694b468b7d7597e3259945148ce6fe39fec4f558dac5de1a554b1508169",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinymemory-module-1.16.1-macos-26-arm64.tar.gz",
            sha256: "dcd45c7e030ef01d6a48174c57aea48adfebb221144fb579c18180f068ec13df",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinymemory-module-1.16.1-macos-26-x86_64.tar.gz",
            sha256: "e2cdc685ecb24ad83ab98a1ce88be4b2e563535219eee8609a91ec1bcc5ac490",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinymemory-module-1.16.1-macos-15-arm64.tar.gz",
            sha256: "fd3c002707011ab594b3ac73114230aed570a6ebc9215e7c915d3024b74d8a19",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinymemory-module-1.16.1-macos-15-x86_64.tar.gz",
            sha256: "63030e318b20a410ea5f0dc8addd6c76a7f1848a45c8badb4e2c96f3f8a0539a",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinymemory-module-1.16.1-windows-2025-x86_64.zip",
            sha256: "94f902a44928d4485a62b5c7531dd91985faa3da466748777d8f1c0f3227eeaa",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinymemory-module-1.16.1-windows-2022-x86_64.zip",
            sha256: "d0f1aea80b3ee1b96eeeb1663b45ffdfd5ca32e09ce9b8653864c8fafc49e0b5",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinymemory-module-1.16.1-windows-11-arm64.zip",
            sha256: "3c287618965a203d1165490ca05693315e523008ad94acaafacb6c2dc998b041",
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
    version: "0.3.4",
    release_url: "https://github.com/tinyhumansai/tinyjuice/releases/tag/v0.3.4",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinyjuice-module-0.3.4-ubuntu-24.04-x86_64.tar.gz",
            sha256: "bcde1fed2e9ad31686334c3f35d83ac8ccf8c6a6cc66ff51e22a24df7d8a925b",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinyjuice-module-0.3.4-ubuntu-24.04-arm64.tar.gz",
            sha256: "306c01bd9180b15c112065ee5d9cf126f0deaa07815551cd92534815158cb4cb",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinyjuice-module-0.3.4-ubuntu-22.04-x86_64.tar.gz",
            sha256: "12c757667a635f97338501f5d0844986fdf1d2cb486e4a54cab970b4f4d4f85e",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinyjuice-module-0.3.4-ubuntu-22.04-arm64.tar.gz",
            sha256: "02b1b2174d3ff5e640774b35c9a67f2e40fc7cc631469471949cd72d01f3fd23",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinyjuice-module-0.3.4-macos-26-arm64.tar.gz",
            sha256: "6caff6e1ffdb8b7b9cbd73b3edd759401872d07ef595c7224f00b6cd5cd676f1",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinyjuice-module-0.3.4-macos-26-x86_64.tar.gz",
            sha256: "caeec46484a36c810f40c0cabc593391027751b6e70cb7e78b2bcc81c032f50c",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinyjuice-module-0.3.4-macos-15-arm64.tar.gz",
            sha256: "640fb8cf7236792d78b5acae6a2835b2eca77f77ecb805fd7541f60fc8048b0c",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinyjuice-module-0.3.4-macos-15-x86_64.tar.gz",
            sha256: "82df562bfc1fd6444cd1b3d06e136c960e42d150e9567ca7e5c20cc5882e4909",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinyjuice-module-0.3.4-windows-2025-x86_64.zip",
            sha256: "754fffdd261637c37475ffb0f3c6577d8643bf020b23218508871db47b89159f",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinyjuice-module-0.3.4-windows-2022-x86_64.zip",
            sha256: "1d187d83662d2e7d1e2ff0af533de81128af55828d95cd3604e609f7d1bb7e19",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinyjuice-module-0.3.4-windows-11-arm64.zip",
            sha256: "fbe1d5fa00eb5097fccc6d2e40b3476a5b5d7972accd0c6e34bfb63dfbf96726",
        },
    ],
    load: LoadPolicy::Lazy,
};
