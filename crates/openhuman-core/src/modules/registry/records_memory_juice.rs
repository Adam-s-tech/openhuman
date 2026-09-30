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
