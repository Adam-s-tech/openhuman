//! Registry records for the `tinymemory` and `tinyjuice` modules.

use crate::modules::types::{LoadPolicy, ModuleRecord, PlatformAsset};

/// The complete TinyMemory engine, loaded eagerly so its capabilities are
/// available when the kernel assembles its RPC and tool surfaces.
pub(crate) const TINYMEMORY: ModuleRecord = ModuleRecord {
    id: "tinymemory",
    description: "Local memory engine: store, ranked recall, and portable export",
    bus_name: "ai.tinyhumans.tinymemory.Memory",
    object_path: "/ai/tinyhumans/tinymemory/Memory",
    version: "1.16.2",
    release_url: "https://github.com/tinyhumansai/tinymemory/releases/tag/v1.16.2",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinymemory-module-1.16.2-ubuntu-24.04-x86_64.tar.gz",
            sha256: "2f825f74757292981c5b5459f1c27c67f705d7e745e67a0190bfcb3f1a249824",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinymemory-module-1.16.2-ubuntu-24.04-arm64.tar.gz",
            sha256: "58a524b7e1af0de31894d5a11f2de180e1fcb83bdcdb3adc4d34749dcfae0603",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinymemory-module-1.16.2-ubuntu-22.04-x86_64.tar.gz",
            sha256: "dd6aed6050e6acdd762583bcc44030f967f56e9022c9cad4b29d935eec4af78c",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinymemory-module-1.16.2-ubuntu-22.04-arm64.tar.gz",
            sha256: "0ac2c4ceaa5f3bc9ff5d3ad90538a241eb45908f7f891351d01308a17713b0aa",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinymemory-module-1.16.2-macos-26-arm64.tar.gz",
            sha256: "23af88b663dde84c8ca733882ea88f707dc7ef67e8af214f5d7ed6d3c3d58eb1",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinymemory-module-1.16.2-macos-26-x86_64.tar.gz",
            sha256: "108db97ab8ce37d8b62dea2c5113f1ede0374c51275e9b7fe63b6ac38a80acee",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinymemory-module-1.16.2-macos-15-arm64.tar.gz",
            sha256: "b9cc73fec722509c7d76a10c414646a2e557540d8242804c61b807e628cdfee5",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinymemory-module-1.16.2-macos-15-x86_64.tar.gz",
            sha256: "626fdedd8d35d6e12cee84473294d8700aa418cec24ba796802558d894c3a19e",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinymemory-module-1.16.2-windows-2025-x86_64.zip",
            sha256: "adeeda66a1903c08c3e71b5b4aecfc9d0548dace499f9f907dbf295a20a662e4",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinymemory-module-1.16.2-windows-2022-x86_64.zip",
            sha256: "9ae15a8256715f253a0fe2544734c430df4b12a4d1e7d33549e80039f5844424",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinymemory-module-1.16.2-windows-11-arm64.zip",
            sha256: "6e2e9208f55912df49a5231b68aa5c45a58108ee3da4b0fe0d24a8a75fea3a96",
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
