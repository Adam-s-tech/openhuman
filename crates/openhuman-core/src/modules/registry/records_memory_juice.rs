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
    version: "0.5.2",
    release_url: "https://github.com/tinyhumansai/tinyjuice/releases/tag/v0.5.2",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinyjuice-module-0.5.2-ubuntu-24.04-x86_64.tar.gz",
            sha256: "970b4ad9c5d5380ffa16b1a34fff79717a4633b970cd8220c62d7a4b52c9d3d7",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinyjuice-module-0.5.2-ubuntu-24.04-arm64.tar.gz",
            sha256: "92fb9d3b5fed037d2569f8e999ba1f15336a343ca556f262f3e5d2555a2907a6",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinyjuice-module-0.5.2-ubuntu-22.04-x86_64.tar.gz",
            sha256: "1a9bc9479ce51c87d82a6f145408cfcbe7d06de4613b0ca7cec1fd04bbc2b5b5",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinyjuice-module-0.5.2-ubuntu-22.04-arm64.tar.gz",
            sha256: "84108c6a8570dea065fd0d7cebf2cf8c257835cc4601da6167a5a1980664006e",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinyjuice-module-0.5.2-macos-26-arm64.tar.gz",
            sha256: "5f8f6381316aaf854ef9ef9a9a29f65dc47889cccfcceb435e2a3bce93a3ab9c",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinyjuice-module-0.5.2-macos-26-x86_64.tar.gz",
            sha256: "2c68408c10f04f4d9c6a8f77379110cf71164c644a3599da85e5885fc525905c",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinyjuice-module-0.5.2-macos-15-arm64.tar.gz",
            sha256: "94eef6b798057f33522d2138f28306dec3e25fd9dc23ae89d0d2ac6ebdc61397",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinyjuice-module-0.5.2-macos-15-x86_64.tar.gz",
            sha256: "652234406624f8c45645227b3271fc83a4bf8efbbd40a4e861cd54d8902ade89",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinyjuice-module-0.5.2-windows-2025-x86_64.zip",
            sha256: "d0ac921ac3cb20c1a69966ea0a2957d78eb1afd73437b3873be7c12652e069e7",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinyjuice-module-0.5.2-windows-2022-x86_64.zip",
            sha256: "7882c325adaa9c4f104edaee8e9bcba41642c3f88266ceb471ec3e3e6094ee04",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinyjuice-module-0.5.2-windows-11-arm64.zip",
            sha256: "bb076acb075e6dca448377e10948ffd3401984619eb281d71b14fb934732ee3f",
        },
    ],
    load: LoadPolicy::Lazy,
};
