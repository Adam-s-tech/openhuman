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
    version: "0.6.0",
    release_url: "https://github.com/tinyhumansai/tinyjuice/releases/tag/v0.6.0",
    assets: &[
        PlatformAsset {
            host_key: "ubuntu-24.04-x86_64",
            archive: "tinyjuice-module-0.6.0-ubuntu-24.04-x86_64.tar.gz",
            sha256: "91ebe1a672d326bfdc92c195f7e5db6189e6e9e10e4f4fe0852219c12a86ae17",
        },
        PlatformAsset {
            host_key: "ubuntu-24.04-arm64",
            archive: "tinyjuice-module-0.6.0-ubuntu-24.04-arm64.tar.gz",
            sha256: "78c80c6267eeb85e6f1f37e0c6d574b4362ad071745dc97477f6a44e81a14217",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-x86_64",
            archive: "tinyjuice-module-0.6.0-ubuntu-22.04-x86_64.tar.gz",
            sha256: "25f7c851a7f2dc5d746ee32a632863b783d76deee6ab0d786ea406238a33f27d",
        },
        PlatformAsset {
            host_key: "ubuntu-22.04-arm64",
            archive: "tinyjuice-module-0.6.0-ubuntu-22.04-arm64.tar.gz",
            sha256: "29accacc64e87ce02ab8d56319149e2e91bf7a40965dbabbb537f3b648b77207",
        },
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinyjuice-module-0.6.0-macos-26-arm64.tar.gz",
            sha256: "d37dc1f5cc370e6c4d1dc3580bc90bc055c18448e4de08f2149085aa721120c6",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinyjuice-module-0.6.0-macos-26-x86_64.tar.gz",
            sha256: "3e5a08d16d8bdbb74e1ddb51adee53fc78b667ba68396625bc7de322239b9a83",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinyjuice-module-0.6.0-macos-15-arm64.tar.gz",
            sha256: "00d67a7031b4a89c59950da2bfc1461b99b44247df9a026231ffef1af1ba299d",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinyjuice-module-0.6.0-macos-15-x86_64.tar.gz",
            sha256: "077d8a501f915bb0ddab96b5cf21d202bf849b343dccab9b4b3f287fbaf0c50a",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinyjuice-module-0.6.0-windows-2025-x86_64.zip",
            sha256: "a390bca33c1e8de560277f8041ac51bb317bc066f901424a8047a079e389f587",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinyjuice-module-0.6.0-windows-2022-x86_64.zip",
            sha256: "4e6d3f621ed98a4fb8b689fe8a84c34e84cadb4a93e18e2b49d23daf92fc408c",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinyjuice-module-0.6.0-windows-11-arm64.zip",
            sha256: "9ca0662f9eaa17ef8b0ab509b747efc3e9bcbfe2242dc12475d41a5b97364a58",
        },
    ],
    load: LoadPolicy::Lazy,
};
