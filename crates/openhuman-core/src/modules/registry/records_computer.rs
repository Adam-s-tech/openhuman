//! Native computer-use module (desktop and browser control). Published digests are added only from its release manifest.

use crate::modules::types::{LoadPolicy, ModuleRecord, PlatformAsset};
use tinycomputer_bus::names;

pub(crate) const TINYCOMPUTER: ModuleRecord = ModuleRecord {
    id: "tinycomputer",
    description: "Permission-aware desktop and browser observation and control",
    bus_name: names::INTERFACE,
    object_path: names::OBJECT_PATH,
    version: "0.7.0",
    release_url: "https://github.com/tinyhumansai/tinycomputer/releases/tag/v0.7.0",
    // Verbatim from the published tinycomputer v0.7.0 checksum.toml. Linux remains outside
    // the initial product surface; this registry admits macOS and Windows.
    assets: &[
        PlatformAsset {
            host_key: "macos-26-arm64",
            archive: "tinycomputer-0.7.0-macos-26-arm64.tar.gz",
            sha256: "5b529cbf4b3403fe49b6a49ba3774e394cda54ee9a2fc44e8b536bc025ec7ba1",
        },
        PlatformAsset {
            host_key: "macos-26-x86_64",
            archive: "tinycomputer-0.7.0-macos-26-x86_64.tar.gz",
            sha256: "66663fae7cedb4be5da40a86466bbe08aede522a7446a1652534e0d3f8832cf6",
        },
        PlatformAsset {
            host_key: "macos-15-arm64",
            archive: "tinycomputer-0.7.0-macos-15-arm64.tar.gz",
            sha256: "da4459dd09764d610ca576ec2f1c89326d4c105e2da8adc4542901c1d3dae1d2",
        },
        PlatformAsset {
            host_key: "macos-15-x86_64",
            archive: "tinycomputer-0.7.0-macos-15-x86_64.tar.gz",
            sha256: "aff3bbcf6074790172237d14acc49169808a758eea891c52ccbae8df8376e276",
        },
        PlatformAsset {
            host_key: "windows-2025-x86_64",
            archive: "tinycomputer-0.7.0-windows-2025-x86_64.zip",
            sha256: "f1148d0a934ebba427fee06dc8ebd0c3ef5ca6198e627515cec40ce7418ddc9d",
        },
        PlatformAsset {
            host_key: "windows-2022-x86_64",
            archive: "tinycomputer-0.7.0-windows-2022-x86_64.zip",
            sha256: "03d12601e74800619345f2b0209c46f775dd5d7dc94005e454788717a451892d",
        },
        PlatformAsset {
            host_key: "windows-11-arm64",
            archive: "tinycomputer-0.7.0-windows-11-arm64.zip",
            sha256: "2258000187708873d2e70cb820ca414ee3e525d6139ae691e8593f3ef4d50d5a",
        },
    ],
    load: LoadPolicy::Lazy,
};
